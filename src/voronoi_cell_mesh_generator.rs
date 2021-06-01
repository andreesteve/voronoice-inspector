use std::iter::once;

use bevy::{
    prelude::*,
    render::{mesh::{Indices, Mesh}, pipeline::PrimitiveTopology},
};
use voronoice::Point;
use crate::utils::point_to_f32_vec;

use super::{
    utils,
    VoronoiCell,
    into_triangle_list::*,
};

pub struct VoronoiCellMeshGenerator<'a> {
    pub cell: &'a VoronoiCell<'a>,
    pub coloring: fn(usize) -> Color
}

impl VoronoiCellMeshGenerator<'_> {
    pub fn build_voronoi_mesh(&self) -> Mesh {
        let vertices: Vec<Point> = self.cell.iter_vertices().cloned().collect();
        let mut positions: Vec<[f32; 3]> = utils::to_f32_vec(&vertices);
        // insert site in the begining so we can fan around it
        positions.insert(0, point_to_f32_vec(self.cell.site_position()));

        let num_of_vertices = positions.len();
        let normals: Vec<[f32; 3]> = vec![[0.0, 1.0, 0.0]; num_of_vertices];
        let uvs: Vec<[f32; 2]> = vec![[0.0, 0.0]; num_of_vertices];
        let colors: Vec<[f32; 3]> = (0..num_of_vertices)
            .map(self.coloring)
            .map(utils::color_to_f32_vec)
            .collect();
        let indices = self.build_voronoi_cell_index_buffer(num_of_vertices as u32);

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(Indices::U32(indices)));
        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.set_attribute("Vertex_Color", colors);
        mesh
    }

    fn build_voronoi_cell_index_buffer(&self, num_of_vertices: u32) -> Vec<u32> {
        (0..num_of_vertices)
            .chain(once(1)) // add first cell vertex (not the 0 which is the site position) to the end so it gets a triangle fanned with the last record
            .into_triangle_list()
            .collect::<Vec<u32>>()
    }
}