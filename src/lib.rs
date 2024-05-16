use std::{cell::RefCell, fmt::Display, path::Path, rc::Rc};

use image::{Rgba, RgbaImage};

type LinkNode = Option<Rc<RefCell<Node>>>;
type LinkElement = Option<Rc<RefCell<(Option<Rgba<u8>>, u32, u32, u32, u32)>>>;

//a faire peut changer la façon dont on stock la hauteur initial => mettre une autre info dans les elements des noeuds ?
#[derive(Debug)]
pub struct Node {
    pub no: LinkNode,
    pub ne: LinkNode,
    pub el: LinkElement,
    pub se: LinkNode,
    pub so: LinkNode,
}
#[derive(Debug)]
pub struct QuadTree {
    pub root: LinkNode,
    pub init_height: isize,
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.el.as_ref().unwrap().borrow().0.is_some() {
            let pixel: String =
                rgba_to_hex(&self.el.as_ref().unwrap().borrow().0.as_ref().unwrap());
            write!(f, "{}", pixel)
        } else {
            let no: String = self
                .no
                .as_ref()
                .map_or(String::new(), |p| format!("{}", p.borrow()));
            let ne: String = self
                .ne
                .as_ref()
                .map_or(String::new(), |p| format!("{}", p.borrow()));
            let se: String = self
                .se
                .as_ref()
                .map_or(String::new(), |p| format!("{}", p.borrow()));
            let so: String = self
                .so
                .as_ref()
                .map_or(String::new(), |p| format!("{}", p.borrow()));
            write!(f, "({} {} {} {})", no, ne, se, so)
        }
    }
}
impl Display for QuadTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.root.as_ref().unwrap().borrow().fmt(f)
    }
}
fn rgba_to_hex(rgba: &Rgba<u8>) -> String {
    let int = 256 * 256 * 256 * rgba.0[0] as u32
        + 256 * 256 * rgba.0[1] as u32
        + 256 * rgba.0[2] as u32
        + rgba.0[3] as u32;
    format!("{int:08x}")
}
fn aux_height(node: &LinkNode) -> isize {
    if node.is_none() {
        -1
    } else {
        let n = node.as_ref().unwrap().borrow();
        1 + aux_height(&n.no)
            .max(aux_height(&n.ne))
            .max(aux_height(&n.se))
            .max(aux_height(&n.so))
    }
}

fn aux_initialize(image: &RgbaImage, x0: u32, y0: u32, x1: u32, y1: u32) -> LinkNode {
    if x0 == x1 && y0 == y1 {
        Some(Rc::new(RefCell::new(Node {
            no: None,
            ne: None,
            el: Some(Rc::new(RefCell::new((
                Some(*image.get_pixel(x0, y0)),
                x0,
                y0,
                x0,
                y0,
            )))),
            se: None,
            so: None,
        })))
    } else {
        let mx = (x0 + x1) / 2;
        let my = (y0 + y1) / 2;

        Some(Rc::new(RefCell::new(Node {
            no: aux_initialize(image, x0, y0, mx, my),
            ne: aux_initialize(image, mx + 1, y0, x1, my),
            el: Some(Rc::new(RefCell::new((None, x0, y0, x1, y1)))),
            se: aux_initialize(image, mx + 1, my + 1, x1, y1),
            so: aux_initialize(image, x0, my + 1, mx, y1),
        })))
    }
}

fn aux_byte_size(node: &LinkNode) -> usize {
    if node.is_none() {
        0
    } else {
        let n: std::cell::Ref<Node> = node.as_ref().unwrap().borrow();
        aux_byte_size(&n.no)
            + aux_byte_size(&n.ne)
            + aux_byte_size(&n.se)
            + aux_byte_size(&n.so)
            + if n.el.is_none() {
                0
            } else {
                std::mem::size_of_val(n.el.as_ref().unwrap())
            }
    }
}

fn aux_is_leaf(node: &LinkNode) -> bool {
    if node.is_none() {
        false
    } else {
        node.as_ref()
            .unwrap()
            .borrow()
            .el
            .as_ref()
            .unwrap()
            .borrow()
            .0
            .is_some()
    }
}

fn aux_lossless_compression(node: &LinkNode) {
    if node.is_some() {
        {
            let mut n: std::cell::RefMut<Node> = node.as_ref().unwrap().borrow_mut();
            if aux_is_leaf(&n.no) && aux_is_leaf(&n.ne) && aux_is_leaf(&n.se) && aux_is_leaf(&n.so)
            {
                let no_el: Option<Rc<RefCell<(Option<Rgba<u8>>, u32, u32, u32, u32)>>> =
                    n.no.as_ref().unwrap().borrow().el.clone();
                let ne_el: Option<Rc<RefCell<(Option<Rgba<u8>>, u32, u32, u32, u32)>>> =
                    n.ne.as_ref().unwrap().borrow().el.clone();
                let se_el: Option<Rc<RefCell<(Option<Rgba<u8>>, u32, u32, u32, u32)>>> =
                    n.se.as_ref().unwrap().borrow().el.clone();
                let so_el: Option<Rc<RefCell<(Option<Rgba<u8>>, u32, u32, u32, u32)>>> =
                    n.so.as_ref().unwrap().borrow().el.clone();
                if no_el == ne_el && ne_el == se_el && se_el == so_el {
                    n.el = Some(Rc::new(RefCell::new((
                        no_el.as_ref().unwrap().borrow().0,
                        no_el.as_ref().unwrap().borrow().1,
                        no_el.as_ref().unwrap().borrow().2,
                        no_el.as_ref().unwrap().borrow().3,
                        no_el.as_ref().unwrap().borrow().4,
                    ))));

                    n.no = None;
                    n.ne = None;
                    n.se = None;
                    n.so = None;
                }
            }
            aux_lossless_compression(&n.no);
            aux_lossless_compression(&n.ne);
            aux_lossless_compression(&n.se);
            aux_lossless_compression(&n.so);
        }
    }
}
fn average_color(colors: &Vec<Rgba<u8>>) -> Rgba<u8> {
    let sum_channel = colors.iter().fold((0, 0, 0, 0), |acc, elm| {
        (
            acc.0 + elm.0[0] as u32,
            acc.1 + elm.0[1] as u32,
            acc.2 + elm.0[2] as u32,
            acc.3 + elm.0[3] as u32,
        )
    });
    let n = colors.len() as u32;
    Rgba([
        (sum_channel.0 / n) as u8,
        (sum_channel.1 / n) as u8,
        (sum_channel.2 / n) as u8,
        (sum_channel.3 / n) as u8,
    ])
}
fn aux_average_compression(node: &LinkNode) {
    if node.is_some() {
        {
            let mut n: std::cell::RefMut<Node> = node.as_ref().unwrap().borrow_mut();
            if aux_is_leaf(&n.no) && aux_is_leaf(&n.ne) && aux_is_leaf(&n.se) && aux_is_leaf(&n.so)
            {
                let no_el: Option<Rc<RefCell<(Option<Rgba<u8>>, u32, u32, u32, u32)>>> =
                    n.no.as_ref().unwrap().borrow().el.clone();
                let ne_el: Option<Rc<RefCell<(Option<Rgba<u8>>, u32, u32, u32, u32)>>> =
                    n.ne.as_ref().unwrap().borrow().el.clone();
                let se_el: Option<Rc<RefCell<(Option<Rgba<u8>>, u32, u32, u32, u32)>>> =
                    n.se.as_ref().unwrap().borrow().el.clone();
                let so_el: Option<Rc<RefCell<(Option<Rgba<u8>>, u32, u32, u32, u32)>>> =
                    n.so.as_ref().unwrap().borrow().el.clone();
                let v: Vec<Rgba<u8>> = vec![
                    *no_el.as_ref().unwrap().borrow().0.as_ref().unwrap(),
                    *ne_el.as_ref().unwrap().borrow().0.as_ref().unwrap(),
                    *se_el.as_ref().unwrap().borrow().0.as_ref().unwrap(),
                    *so_el.as_ref().unwrap().borrow().0.as_ref().unwrap(),
                ];
                let average_color: Rgba<u8> = average_color(&v);
                let el_clone = n.el.clone();
                n.el = Some(Rc::new(RefCell::new((
                    Some(average_color),
                    el_clone.as_ref().unwrap().borrow().1,
                    el_clone.as_ref().unwrap().borrow().2,
                    el_clone.as_ref().unwrap().borrow().3,
                    el_clone.as_ref().unwrap().borrow().4,
                ))));
                n.no = None;
                n.ne = None;
                n.se = None;
                n.so = None;
            }
            aux_average_compression(&n.no);
            aux_average_compression(&n.ne);
            aux_average_compression(&n.se);
            aux_average_compression(&n.so);
        }
    }
}
fn generate_image(
    node: &LinkNode,
    path: &Path,
    init_height: isize,
) -> Result<(), image::ImageError> {
    fn aux(image: &mut RgbaImage, node: &LinkNode) {
        if node.is_none() {
        } else if aux_is_leaf(node) {
            let (c, x0, y0, x1, y1) = *node
                .as_ref()
                .unwrap()
                .borrow()
                .el
                .as_ref()
                .unwrap()
                .borrow();
            for x in x0..=x1 {
                for y in y0..=y1 {
                    image.put_pixel(x, y, *c.as_ref().unwrap());
                }
            }
        } else {
            let no: &Option<Rc<RefCell<Node>>> = &node.as_ref().unwrap().borrow().no;
            let ne: &Option<Rc<RefCell<Node>>> = &node.as_ref().unwrap().borrow().ne;
            let se: &Option<Rc<RefCell<Node>>> = &node.as_ref().unwrap().borrow().se;
            let so: &Option<Rc<RefCell<Node>>> = &node.as_ref().unwrap().borrow().so;
            aux(image, no);
            aux(image, ne);
            aux(image, se);
            aux(image, so);
        }
    }

    let mut img: image::ImageBuffer<Rgba<u8>, Vec<u8>> =
        RgbaImage::new(init_height as u32, init_height as u32);
    aux(&mut img, node);
    img.save(path)
}
impl QuadTree {
    pub fn new() -> Self {
        QuadTree {
            root: None,
            init_height: 0,
        }
    }
    pub fn height(&self) -> isize {
        aux_height(&self.root)
    }
    pub fn initialize(image: &RgbaImage) -> QuadTree {
        Self {
            root: aux_initialize(image, 0, 0, image.width() - 1, image.height() - 1),
            init_height: image.width() as isize,
        }
    }
    pub fn byte_size(&self) -> usize {
        aux_byte_size(&self.root)
    }
    pub fn lossless_compression(&self) {
        aux_lossless_compression(&self.root);
    }
    pub fn average_compression(&self) {
        aux_average_compression(&self.root);
    }
    pub fn generate_image(&self, path: &Path) -> Result<(), image::ImageError> {
        generate_image(&self.root, path, self.init_height)
    }
}
