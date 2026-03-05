use anyhow::Ok;

pub struct PageHeader {
    pub size: u8,
    pub cell_count: u16
}

pub struct Page {
    pub header: PageHeader,
    pub cells: Vec<Cell>
}

pub struct Cell {
    pub pointer: u16
}

impl Page {
    pub fn get_cell_count(&self) -> anyhow::Result<u16> {
        Ok(self.header.cell_count)
    }

    pub fn get_table_names(&self) {
        for i in self.cells.iter() {
            println!("pointer {}", i.pointer.to_string());
        }
    }
}