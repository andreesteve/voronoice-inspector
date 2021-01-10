use bevy::{
    prelude::*,
    render::{mesh::{Indices, Mesh}, pipeline::PrimitiveTopology},
};
use utils::{into_line_list_wrap, into_line_list};
use super::{
    utils,
    Voronoi,
    into_triangle_list::*,
};

pub struct VoronoiMeshGenerator<'a> {
    pub voronoi: &'a Voronoi,
    pub coloring: fn(usize) -> Color,
    pub topology: PrimitiveTopology
}

impl VoronoiMeshGenerator<'_> {
    #[allow(dead_code)]
    pub fn build_circumcenters_mesh(&self) -> Mesh {
        let positions: Vec<[f32; 3]> = utils::to_f32_vec(self.voronoi.vertices());
        let num_of_vertices = positions.len();
        let normals: Vec<[f32; 3]> = vec![[0.0, 1.0, 0.0]; num_of_vertices];
        let uvs: Vec<[f32; 2]> = vec![[0.0, 0.0]; num_of_vertices];
        let colors: Vec<[f32; 3]> = (0..num_of_vertices)
            .map(self.coloring)
            .map(utils::color_to_f32_vec)
            .collect();

        let indices = (0..num_of_vertices).map(|e| e as u32).collect();

        let mut mesh = Mesh::new(self.topology);
        mesh.set_indices(Some(Indices::U32(indices)));
        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.set_attribute("Vertex_Color", colors);
        mesh
    }

    pub fn build_delauney_mesh(&self) -> Mesh {
        let positions: Vec<[f32; 3]> = utils::to_f32_vec(self.voronoi.sites());
        let num_of_vertices = positions.len();
        let normals: Vec<[f32; 3]> = vec![[0.0, 1.0, 0.0]; num_of_vertices];
        let uvs: Vec<[f32; 2]> = vec![[0.0, 0.0]; num_of_vertices];
        let colors: Vec<[f32; 3]> = (0..num_of_vertices)
            .map(self.coloring)
            .map(utils::color_to_f32_vec)
            .collect();

        let mut indices: Vec<u32> = vec![];
        let triangles = self.voronoi.delauney_triangles();
        for t in 0..(triangles.len() / 3) {
            indices.push(triangles[3 * t] as u32);
            indices.push(triangles[3 * t + 1] as u32);

            indices.push(triangles[3 * t + 1] as u32);
            indices.push(triangles[3 * t + 2] as u32);

            indices.push(triangles[3 * t + 2] as u32);
            indices.push(triangles[3 * t] as u32);
        }

        let mut mesh = Mesh::new(self.topology);
        mesh.set_indices(Some(Indices::U32(indices)));
        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.set_attribute("Vertex_Color", colors);
        mesh
    }

    pub fn build_voronoi_mesh(&self) -> Mesh {
        let positions: Vec<[f32; 3]> = utils::to_f32_vec(self.voronoi.vertices());
        let num_of_vertices = positions.len();
        let normals: Vec<[f32; 3]> = vec![[0.0, 1.0, 0.0]; num_of_vertices];
        let uvs: Vec<[f32; 2]> = vec![[0.0, 0.0]; num_of_vertices];
        let colors: Vec<[f32; 3]> = (0..num_of_vertices)
            .map(self.coloring)
            .map(utils::color_to_f32_vec)
            .collect();
        let indices = self.build_voronoi_cell_index_buffer();

        let mut mesh = Mesh::new(self.topology);
        mesh.set_indices(Some(Indices::U32(indices)));
        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.set_attribute("Vertex_Color", colors);
        mesh
    }

    fn build_voronoi_cell_index_buffer(&self) -> Vec<u32> {
        match self.topology {
            PrimitiveTopology::LineList => {
                self.voronoi.iter_cells()
                .flat_map(|c| into_line_list_wrap(c.iter_triangles()))
                .map(|t| t as u32)
                .collect::<Vec<u32>>()
            },

            PrimitiveTopology::PointList | PrimitiveTopology::TriangleList => {
                // if cells on hull are not closed, they will not render correctly in this mode
                self.voronoi.iter_cells()
                    .flat_map(|c| into_line_list(c.iter_triangles()))
                    .map(|t| t as u32)
                    .into_triangle_list()
                    .collect::<Vec<u32>>()
            },

            _ => panic!("Topology {:?} not supported", self.topology)
        }
    }
}