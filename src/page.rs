use anyhow::Ok;

pub struct PageHeader {
    pub cell_count: u16
}

pub struct Page {
    pub header: PageHeader
}

impl Page {
    pub fn get_cell_count(&self) ->anyhow::Result<u16> {
        Ok(self.header.cell_count)
    }
}