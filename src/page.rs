pub struct PageHeader {
    pub size: usize,
    pub cell_count: u16
}

pub struct Page {
    pub header: PageHeader,
    pub cells: Vec<Cell>
}

impl Page {
    pub fn get(&self, n: usize) -> Option<&Cell> {
        self.cells.get(n)
    }
}

pub struct Cell {
    //pub pointer: u16,
    pub payload: Vec<u8>
}

impl Page {
    pub fn get_cell_count(&self) -> usize {
        self.header.cell_count as usize
    }
}