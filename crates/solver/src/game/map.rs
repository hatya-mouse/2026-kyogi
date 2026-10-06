use crate::game::{CellId, CellType};

pub struct Map {
    /// The width and height of the map.
    size: (usize, usize),
    /// The main hexagonal map.
    pub cells: Vec<CellType>,
}

impl Map {
    pub fn new(size: (usize, usize), cells: Vec<CellType>) -> Self {
        Self { size, cells }
    }

    /// The width of the map.
    #[inline]
    pub fn width(&self) -> usize {
        self.size.0
    }

    /// The height of the map.
    #[inline]
    pub fn height(&self) -> usize {
        self.size.1
    }

    /// Gets the cell with the given CellId.
    #[inline]
    pub fn get_cell(&self, id: &CellId) -> Option<&CellType> {
        self.cells.get(id.0)
    }

    /// Calculates an CellId from a given column and row on the map.
    #[inline]
    pub fn get_id_from_coord(&self, coord: (usize, usize)) -> CellId {
        CellId::from_coord(coord, self.size.0)
    }

    /// Calculates a column and a row for a given index on the map.
    #[inline]
    pub fn get_coord_from_id(&self, id: CellId) -> (usize, usize) {
        id.to_coord(self.size.0)
    }

    /// Calculates a XYZ coordinate for a given index on the map.
    fn get_xyz_coord_from_id(&self, id: CellId) -> (isize, isize, isize) {
        let (col, row) = self.get_coord_from_id(id);
        let (icol, irow) = (col as isize, row as isize);

        let x = icol;
        let z = irow - (icol - (icol & 1)) / 2;
        let y = -x - z;

        (x, y, z)
    }

    /// Calculates a distance between two points on the map.
    pub fn distance(&self, start: CellId, end: CellId) -> u32 {
        let start_coord = self.get_xyz_coord_from_id(start);
        let end_coord = self.get_xyz_coord_from_id(end);

        let dx = end_coord.0 - start_coord.0;
        let dy = end_coord.1 - start_coord.1;
        let dz = end_coord.2 - start_coord.2;

        (dx.abs() + dy.abs() + dz.abs()) as u32 / 2
    }

    /// Gets the neighbors of a given index on the map, excluding the impassable tiles.
    pub fn get_neighbors(&self, id: CellId) -> Vec<CellId> {
        let (col, row) = self.get_coord_from_id(id);
        let (icol, irow) = (col as isize, row as isize);

        let offsets = if row % 2 == 0 {
            [(1, 0), (-1, 0), (0, 1), (0, -1), (-1, 1), (-1, -1)]
        } else {
            [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1)]
        };
        offsets
            .iter()
            .map(|(dc, dr)| (icol + dc, irow + dr))
            .filter(|&(c, r)| {
                c >= 0 && r >= 0 && c < self.width() as isize && r < self.height() as isize
            })
            .filter_map(|(c, r)| {
                let id = self.get_id_from_coord((c as usize, r as usize));
                self.get_cell(&id)
                    .and_then(|cell| if cell.is_passable() { Some(id) } else { None })
            })
            .collect()
    }
}
