#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct CollationTable {
    pub page_index: Vec<u16>,
    pub entries: Vec<u64>,
    pub contraction_meta: Vec<ContractionMeta>,
    pub edges: Vec<ContractionEdge>,
    pub weights: Vec<u32>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ContractionMeta {
    pub first_edge: u32,
    pub edge_len: u16,
    pub max_len: u8,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ContractionEdge {
    pub code_point: u32,
    pub next_first_edge: u32,
    pub weight_start: u32,
    pub next_edge_len: u16,
    pub weight_len: u16,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct VariableTable {
    pub page_index: Vec<u16>,
    pub pages: Vec<u64>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct FcdTable {
    pub page_index: Vec<u16>,
    pub pages: Vec<u16>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct DecompTable {
    pub page_index: Vec<u16>,
    pub entries: Vec<u64>,
    pub values: Vec<u32>,
}

impl DecompTable {
    pub fn get(&self, code_point: u32) -> Option<&[u32]> {
        let page = self.page_index[(code_point >> 8) as usize];
        if page == EMPTY_PAGE {
            return None;
        }

        let entry = self.entries[(usize::from(page) << 8) + (code_point & 0xFF) as usize];
        let len = decomp_len(entry);
        if len == 0 {
            return None;
        }

        let start = decomp_start(entry);
        Some(&self.values[start..start + usize::from(len)])
    }
}

impl FcdTable {
    pub fn get(&self, code_point: u32) -> Option<u16> {
        let page = self.page_index[(code_point >> 8) as usize];
        if page == EMPTY_PAGE {
            return None;
        }

        let index = usize::from(page) * 256 + (code_point & 0xFF) as usize;
        let val = self.pages[index];
        (val != 0).then_some(val)
    }
}

impl VariableTable {
    pub fn contains(&self, code_point: u32) -> bool {
        let page = self.page_index[(code_point >> 8) as usize];
        if page == EMPTY_PAGE {
            return false;
        }

        let page_start = usize::from(page) * 4;
        let offset = code_point & 0xFF;
        let word = page_start + (offset >> 6) as usize;
        let bit = offset & 0x3F;
        (self.pages[word] & (1_u64 << bit)) != 0
    }
}

impl CollationTable {
    pub fn entry(&self, code_point: u32) -> u64 {
        let page = self.page_index[(code_point >> 8) as usize] as usize;
        self.entries[(page << 8) + (code_point & 0xFF) as usize]
    }

    pub const fn is_missing(entry: u64) -> bool {
        entry_tag(entry) == ENTRY_MISSING
    }

    pub const fn is_contraction(entry: u64) -> bool {
        entry_tag(entry) == ENTRY_CONTRACTION
    }

    pub fn max_len(&self, entry: u64) -> usize {
        if Self::is_contraction(entry) {
            usize::from(self.contraction_meta(entry).max_len)
        } else {
            1
        }
    }

    pub fn simple_row(&self, entry: u64) -> &[u32] {
        self.weights_slice(entry_start(entry), entry_len(entry))
    }

    pub fn get2(&self, entry: u64, b: u32) -> Option<&[u32]> {
        if !Self::is_contraction(entry) {
            return None;
        }

        let meta = self.contraction_meta(entry);
        let edge = self.find_edge(meta.first_edge, meta.edge_len, b)?;
        self.edge_row(edge)
    }

    pub fn get3(&self, entry: u64, b: u32, c: u32) -> Option<&[u32]> {
        if !Self::is_contraction(entry) {
            return None;
        }

        let meta = self.contraction_meta(entry);
        let edge = self.find_edge(meta.first_edge, meta.edge_len, b)?;
        let edge = self.find_edge(edge.next_first_edge, edge.next_edge_len, c)?;
        self.edge_row(edge)
    }

    fn contraction_meta(&self, entry: u64) -> &ContractionMeta {
        &self.contraction_meta[entry_meta_index(entry)]
    }

    fn find_edge(&self, first_edge: u32, edge_len: u16, cp: u32) -> Option<&ContractionEdge> {
        let start = first_edge as usize;
        let range = &self.edges[start..start + edge_len as usize];

        if edge_len <= 4 {
            return range.iter().find(|edge| edge.code_point == cp);
        }

        let index = range
            .binary_search_by_key(&cp, |edge| edge.code_point)
            .ok()?;
        Some(&range[index])
    }

    fn edge_row(&self, edge: &ContractionEdge) -> Option<&[u32]> {
        if edge.weight_len == 0 {
            return None;
        }

        Some(self.weights_slice(edge.weight_start, edge.weight_len))
    }

    fn weights_slice(&self, start: u32, len: u16) -> &[u32] {
        let start = start as usize;
        &self.weights[start..start + len as usize]
    }
}

const ENTRY_MISSING: u8 = 0;
const ENTRY_CONTRACTION: u8 = 2;
const EMPTY_PAGE: u16 = u16::MAX;

const fn entry_tag(entry: u64) -> u8 {
    (entry & 0b11) as u8
}

const fn entry_len(entry: u64) -> u16 {
    ((entry >> 2) & 0xFFFF) as u16
}

const fn entry_start(entry: u64) -> u32 {
    ((entry >> 18) & 0xFFFF_FFFF) as u32
}

const fn entry_meta_index(entry: u64) -> usize {
    (entry >> 50) as usize
}

const fn decomp_len(entry: u64) -> u16 {
    (entry & 0xFFFF) as u16
}

const fn decomp_start(entry: u64) -> usize {
    (entry >> 16) as usize
}
