use std::slice::from_raw_parts;
use crate::graph::{EdgeData, EdgeHandle, Handle, Header, MSize, VertexHandle, Vertices};
use crate::utils::{handle_from_edge_handle, vertex_handle};

pub struct TreeView<'a, T> {
    pub nodes: &'a mut EdgeData,
    pub values: &'a mut Vertices<T>,
}

const TREE_HEADER_ELEMENTS: MSize = 2;



impl <'a, T> TreeView<'a, T> {
    #[cfg_attr(release, inline(always))]
    pub fn new(edges: &'a mut EdgeData, vertices: &'a mut Vertices<T>) -> Self {
        return TreeView{
            nodes: edges,
            values: vertices,
        }
    }

    pub fn get_children(&self, parent: EdgeHandle) -> &[EdgeHandle] {
        let (header, data) = Header::parse_ptr(&self.nodes.edges, parent);
        let size = unsafe {(*header).len - TREE_HEADER_ELEMENTS} as usize; // Parent and root is not a child
        return unsafe {from_raw_parts(data.add(TREE_HEADER_ELEMENTS as usize), size)};
    }

    #[cfg_attr(release, inline(always))]
    pub fn add_child(&mut self, parent: EdgeHandle, child: EdgeHandle){
        self.nodes.connect(parent, child);
        self.nodes.set(child, parent, 1);
        self.nodes.set(child, self.get_root(parent).edge_handle, 0);
    }

    fn create_vertex(&mut self, val: T) -> Handle {
        self.values.push(val);
        let vertex_handle = VertexHandle((self.values.len() - 1) as MSize);
        let edge_handle = self.nodes.create_edges_entry(0, vertex_handle);
        return Handle{
            edge_handle,
            vertex_handle,
        }
    }
    #[cfg_attr(release, inline(always))]
    pub fn get_root(&self, vertex: EdgeHandle) -> Handle{
        let root = self.nodes.get(vertex, 0);
        return self.nodes.handle_from_edge_handle(root);
    }
    #[cfg_attr(release, inline(always))]

    pub fn get_parent(&self, vertex: EdgeHandle) -> Handle{
        let parent = self.nodes.get(vertex, 1);

        return self.nodes.handle_from_edge_handle(parent);
    }

    #[cfg_attr(release, inline(always))]
    pub fn vertex_handle(&self, edge_handle: EdgeHandle) -> VertexHandle {
        return vertex_handle(&self.nodes.edges, edge_handle);
    }

    #[cfg_attr(release, inline(always))]
    pub fn handle_from_edge_handle(&self, edge_handle: EdgeHandle) -> Handle {
        return handle_from_edge_handle(&self.nodes.edges, edge_handle);
    }

    pub fn create_node(&mut self, val: T) -> Handle {
        let vertex = self.create_vertex(val);

        self.nodes.connect(vertex.edge_handle, vertex.edge_handle); // root
        self.nodes.connect(vertex.edge_handle, EdgeData::NONE); // parent

        return vertex;
    }

    #[cfg_attr(release, inline(always))]
    pub fn create_child(&mut self, parent: EdgeHandle, val: T) -> Handle {
        let child = self.create_node(val);
        self.add_child(parent, child.edge_handle);
        return child;
    }
}