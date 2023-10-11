use super::element_grid::ElementGrid;

/// A 3x3 grid of element grids
/// However, it's a bit complicated because at the top boundary
/// you might encounter a doubling of the grid size, in the case where you are going up
/// a level, that's why there is a t1 and t2.
/// Or at the very top level all the upper levels might be None
/// And going down a layer you might not have a bottom layer, because you might be at the bottom
/// Also going down a layer you may not have a b, because you would only have a bl or br
pub struct ElementGridConvolution {
    t1: Option<ElementGrid>,
    t2: Option<ElementGrid>,
    tl: Option<ElementGrid>,
    tr: Option<ElementGrid>,
    l: ElementGrid,
    r: ElementGrid,
    bl: Option<ElementGrid>,
    b: Option<ElementGrid>,
    br: Option<ElementGrid>,
}

/// We implement IntoIterator for ElementGridConvolution so that we can unpackage it
/// back into a element grid directory
pub struct IntoIter {
    convolution: ElementGridConvolution,
    position: usize,
}

impl Iterator for IntoIter {
    type Item = ElementGrid;

    /// Skip over the None values
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.position += 1;
            let next_item = match self.position {
                1 => self.convolution.t1.take(),
                2 => self.convolution.t2.take(),
                2 => self.convolution.tl.take(),
                3 => self.convolution.tr.take(),
                4 => Some(self.convolution.l),
                5 => Some(self.convolution.r),
                6 => self.convolution.bl.take(),
                7 => self.convolution.b.take(),
                8 => self.convolution.br.take(),
                _ => return None,
            };
            if next_item.is_some() {
                return next_item;
            }
        }
    }
}

impl IntoIterator for ElementGridConvolution {
    type Item = ElementGrid;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            convolution: self,
            position: 0,
        }
    }
}
