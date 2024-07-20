use std::cmp::min;
use std::collections::VecDeque;
use std::mem::{size_of, transmute};
use std::ops::{Index, IndexMut};
use std::slice::{from_raw_parts, from_raw_parts_mut};
use std::thread::available_parallelism;
use crate::traits;
use crate::utils::{handle_from_edge_handle, split_to_parts_mut, vertex_handle};
use crate::views::tree::TreeView;

#[derive(Debug)]
pub enum Error {
    NoHandle,
}

#[derive(PartialEq, Eq)]
pub enum TraverseResult {
    Continue,
    End,
}

#[cfg(msize_type = "u8")]
pub type MSize = u8;

#[cfg(msize_type = "u16")]
pub type MSize = u16;

#[cfg(msize_type = "u32")]
pub type MSize = u32;

#[cfg(msize_type = "usize")]
pub type MSize = usize;

const MSIZE_ALIGN_MASK: usize = size_of::<MSize>() - 1;

#[repr(C)]
pub struct Header{
    pub len: MSize,
    pub reserve: MSize,
    pub visited_flag: MSize,
    pub v_index: VertexHandle,
}

pub struct Vertices<T> {
    data: Vec<T>,
}

pub struct Graph<T> {
    pub vertices: Vertices<T>,
    pub edges: EdgeData,
}


pub struct EdgeData{
    visited_val: MSize, // Val used to mark whether the vertex has been visited
    reserve: usize,
    pub edges: Vec<MSize>,
}
#[derive(Copy, Clone)]
pub struct EdgeHandle(pub MSize);
#[derive(Copy, Clone)]
pub struct VertexHandle(pub MSize);

#[derive(Copy, Clone)]
pub struct Handle {
    pub edge_handle: EdgeHandle,
    pub vertex_handle: VertexHandle,
}

pub fn edge_handle_slice_as_slice(handle: &[EdgeHandle]) -> &[MSize] {
    return unsafe{from_raw_parts(handle.as_ptr() as *const MSize, handle.len())};
}

pub fn edge_handle_slice_as_mut_slice(handle: &mut [VertexHandle]) -> &mut [MSize] {
    return unsafe{from_raw_parts_mut(handle.as_mut_ptr() as *mut MSize, handle.len())};
}

impl Handle {
    #[cfg_attr(release, inline(always))]
    pub fn new(edge_handle: MSize, val_handle: MSize) -> Self {
        return Handle{
            edge_handle: EdgeHandle(edge_handle),
            vertex_handle: VertexHandle(val_handle),
        }
    }
}

impl Header {
    #[cfg_attr(release, inline(always))]
    pub fn parse_ptr_mut (edges: &mut Vec<MSize>, index: EdgeHandle) -> (*mut Self, *mut EdgeHandle) {
        let edges_ptr = edges.as_mut_ptr();
        unsafe{
            let header_ptr = edges_ptr.add(index.0 as usize) as *mut Header;
            let data_ptr = edges_ptr.byte_add(size_of::<Header>()).add(index.0 as usize) as *mut EdgeHandle;
            return (header_ptr, data_ptr);
        }
    }
    #[cfg_attr(release, inline(always))]

    pub fn parse_ptr (edges: &Vec<MSize>, index: EdgeHandle) -> (*const Self, *const EdgeHandle) {
        let edges_ptr = edges.as_ptr();
        unsafe{
            let header_ptr = edges_ptr.add(index.0 as usize) as *const Header;
            let data_ptr = edges_ptr.byte_add(size_of::<Header>()).add(index.0 as usize) as *const EdgeHandle;
            return (header_ptr, data_ptr);
        }
    }

    pub fn parse_mut_header (edges: &mut Vec<MSize>, index: EdgeHandle) -> (&mut Self, &[EdgeHandle]) {
        let edges_ptr = edges.as_mut_ptr();

        // Return as Result instead of panic
        if index.0 as usize >= edges.len() {
            panic!("Index out of bounds");
        }

        unsafe{
            let header_ptr = edges_ptr.add(index.0 as usize) as *mut Header;
            let data_ptr = edges_ptr.byte_add(size_of::<Header>()).add(index.0 as usize) as *const EdgeHandle;
            let data = from_raw_parts(data_ptr, (*header_ptr).reserve as usize);
            return (header_ptr.as_mut().unwrap(), data);
        }
    }
    pub fn parse_mut (edges: &mut Vec<MSize>, index: EdgeHandle) -> (&mut Self, &mut [EdgeHandle]) {
        let edges_ptr = edges.as_mut_ptr();

        // Return as Result instead of panic
        if index.0 as usize >= edges.len() {
            panic!("Index out of bounds");
        }

        unsafe{
            let header_ptr = edges_ptr.add(index.0 as usize) as *mut Header;
            let data_ptr = edges_ptr.byte_add(size_of::<Header>()).add(index.0 as usize) as *mut EdgeHandle;
            let data = from_raw_parts_mut(data_ptr, (*header_ptr).reserve as usize);
            return (header_ptr.as_mut().unwrap(), data);
        }
    }
    pub fn parse (edges: &Vec<MSize>, index: EdgeHandle) -> (&Self, &[EdgeHandle]) {
        // Return as Result instead of panic
        let edges_ptr = edges.as_ptr();

        if index.0 as usize >= edges.len() {
            panic!("Index out of bounds");
        }
        unsafe{
            let header_ptr = edges_ptr.add(index.0 as usize) as *const Header;
            let data_ptr = edges_ptr.byte_add(size_of::<Header>()).add(index.0 as usize) as *const EdgeHandle;
            let data = from_raw_parts(data_ptr, (*header_ptr).len as usize);
            return (header_ptr.as_ref().unwrap(), data);
        }
    }
}
impl<T> Graph<T>{

    pub fn tree_view(&mut self) -> TreeView<T> {
        return TreeView::new(&mut self.edges, &mut self.vertices);
    }

    /// Creates a new graph with the assumption that the usage will be dynamic.
    /// It will create the graph with high reserve count of 50 to avoid reallocations.
    pub fn new_large() -> Self {
        return Graph{
            edges: EdgeData::new_dyn(),
            vertices: Vertices::new(),

        }
    }
    /// Creates a new graph with a custom reserve
    pub fn with_reserve(reserve: usize) -> Self {
        return Graph{
            edges: EdgeData::with_reserve(reserve),
            vertices: Vertices::new(),
        };
    }

    /// Creates a new graph with the assumption that the graph size is known ahead of time. Small reserve count of 5
    pub fn new() -> Self {
        return Graph{
            edges: EdgeData::new(),
            vertices: Vertices::new(),
        };
    }

    pub fn create_and_connect(&mut self, src_edge_handle: EdgeHandle, val: T, edge_count: usize) -> Handle {
        let new_handle = self.create(val, edge_count);
        self.edges.connect(src_edge_handle, new_handle.edge_handle);
        return new_handle
    }
    pub fn create_and_connect_leaf(&mut self, src_edge_handle: EdgeHandle, val: T) -> Handle {
        return self.create_and_connect(src_edge_handle, val, 0);
    }

    pub fn create(&mut self, val: T, edge_count: usize) -> Handle {
        self.vertices.push(val);
        let vertex_handle = VertexHandle((self.vertices.len() - 1) as MSize);
        let edge_handle = self.edges.create_edges_entry(edge_count, vertex_handle);
        return Handle{edge_handle, vertex_handle};
    }
    #[cfg_attr(release, inline(always))]
    pub fn create_leaf(&mut self, val: T) -> Handle {
        return self.create(val, 0)
    }

    pub fn bfs<F>(&mut self, start: EdgeHandle, mut traverse: F)
    where F: FnMut(&mut Self, Handle) -> TraverseResult{
        let start = self.edges.handle_from_edge_handle(start);
        let mut nodes: VecDeque<Handle> = VecDeque::new();

        nodes.push_back(start);
        while nodes.len() > 0 {
            let val = nodes.pop_front().unwrap();
            if traverse(self, val) == TraverseResult::End {
                self.edges.visited_val += 1; // We need a new marker as not whole graph was traversed
                break;
            }
            self.edges.inc_visited_flag(val.edge_handle);
            let data = self.edges.edges(val.edge_handle).unwrap(); //This has to be always valid
            //This has to be always valid
            for next in data {
                let (header, _) = Header::parse(&self.edges.edges, *next);
                if header.visited_flag == self.edges.visited_val {
                    continue;
                }
                let handle = Handle::new((*next).0, header.v_index.0);
                nodes.push_back(handle);
            }
        }
        self.edges.visited_val = 0; // Reset the visited flag as we traversed the whole graph
    }

}


impl <T: Send> traits::Transform<T> for Vertices<T> {
    fn transform(&mut self, transform_fn: fn(&mut [T])) {
        transform_fn(self.data.as_mut_slice());
    }
    fn async_transform(&mut self, transform_fn: fn(&mut [T])) {
        let max_parallelism = available_parallelism().ok().unwrap().get();
        let parallelism_count = min(max_parallelism, self.data.len());
        let parts = split_to_parts_mut(&mut self.data, parallelism_count);

        std::thread::scope(|scope| {
            for part in parts {
                scope.spawn(|| {
                    transform_fn(part);
                });
            }
        });


    }

}
impl <T> Vertices<T>{
    pub fn new() -> Self {
        return Vertices{
            data: Vec::new(),
        }
    }

    #[cfg_attr(release, inline(always))]
    pub fn push(&mut self, val: T) {
        self.data.push(val);
    }
    pub fn len(&self) -> usize {
        return self.data.len();
    }
    #[cfg_attr(release, inline(always))]

    pub fn at(&self, index: MSize) -> &T {
        return &self.data[index as usize];
    }
}

impl <T> Index<VertexHandle> for Vertices<T>{
    type Output = T;
    fn index(&self, index: VertexHandle) -> &Self::Output {
        return &self.data[index.0 as usize];
    }
}

impl <T> IndexMut<VertexHandle> for Vertices<T>{
    fn index_mut(&mut self, index: VertexHandle) -> &mut Self::Output {
        return &mut self.data[index.0 as usize];
    }
}
impl EdgeData {
    pub const NONE: EdgeHandle = EdgeHandle(MSize::MAX);
    const MSIZE_ALIGN_MASK: usize = size_of::<MSize>() - 1;

    /// Creates a new graph with the assumption that the usage will be dynamic.
    /// It will create the graph with high reserve count of 50 to avoid reallocations.
    pub fn new_dyn() -> Self {
        return EdgeData{
            visited_val: 1,
            reserve: 50,
            edges: Vec::new(),
        }
    }
    /// Creates a new graph with a custom reserve
    pub fn with_reserve(capacity: usize) -> Self {
        return EdgeData{
            visited_val: 1,
            reserve: capacity,
            edges: Vec::new(),
        }
    }

    /// Creates a new graph with the assumption that the graph size is known ahead of time. No reserve.
    pub fn new() -> Self {
        return EdgeData{
            visited_val: 1,
            reserve: 0,
            edges: Vec::new(),
        }
    }

    pub fn add_edges(&mut self, edge_handle: EdgeHandle, new_edges: &[EdgeHandle]) {
        let (header, data) = Header::parse_mut(&mut self.edges, edge_handle);
        let new_size = header.len as usize + new_edges.len();

        // TODO return as Result instead of panic!
        if new_size > header.reserve as usize {
            panic!("Edge size is greater than the allocated size");
        }
        let new_data_end = header.len as usize + new_edges.len();

        data[header.len as usize..new_data_end].copy_from_slice(new_edges);
        header.len = new_size as MSize;
    }

    #[cfg_attr(release, inline(always))]
    fn calculate_new_edges_size_abs(&self, size: usize) -> usize {
        let header_size = header_size_in_msize_units();
        return self.edges.len() + self.reserve + header_size + size;
    }
    pub fn create_edges_entry(&mut self, size: usize, vertex_handle: VertexHandle) -> EdgeHandle{
        let offset = self.edges.len();
        let val = self.calculate_new_edges_size_abs(size);
        self.edges.resize_with(val, Default::default);
        unsafe{
            let header_ptr = self.edges.as_mut_ptr().add(offset) as *mut Header;
            (*header_ptr).reserve = self.reserve as MSize + size as MSize;
            (*header_ptr).v_index = vertex_handle;

        }
        return EdgeHandle(offset as MSize);
    }

    //TODO Add checks for unsafe
    pub fn disconnect(&mut self, src: EdgeHandle, vertex: EdgeHandle) {
        let edges_index = src;
        let (header, data) = Header::parse_ptr_mut(&mut self.edges, edges_index);

        unsafe {
            let mut iter = data;
            let end = iter.add((*header).len as usize);
             while iter != end{
                if (*iter).0 == vertex.0{
                    *iter = *end.offset(-1 ); // Swap the last element for the empty one
                    (*header).len -= 1;
                    break;
                }
                iter = iter.offset(1);
            }
        }
    }

    #[cfg_attr(release, inline(always))]
    pub fn set(&mut self, src: EdgeHandle, edge_handle: EdgeHandle, edge_offset: usize){
        let edges = self.edges_mut(src).expect("Vertex not found");
        edges[edge_offset] = edge_handle;
    }

    #[cfg_attr(release, inline(always))]
    pub fn get(&self, vertex: EdgeHandle, offset: usize) -> EdgeHandle{
        return EdgeHandle(self.edges[vertex.0 as usize + header_size_in_msize_units() + offset]);
    }


    #[cfg_attr(release, inline(always))]
    pub fn len(&self, vertex: EdgeHandle) -> MSize {
        let (header, _) = Header::parse(&self.edges, vertex);
        return header.len;
    }

    #[cfg_attr(release, inline(always))]
    pub fn connect(&mut self, from: EdgeHandle, to: EdgeHandle) {
        self.add_edges(from, &[to]);
    }
    #[cfg_attr(release, inline(always))]
    pub fn capacity(&self) -> usize {
        return self.edges.len();
    }

    pub fn reserve(&mut self, edge_handle: EdgeHandle) -> MSize {
        unsafe {
            let header = &self.edges[edge_handle.0 as usize] as *const MSize;
            let header_ptr: *const Header = transmute(header);
            let reserve = (*header_ptr).reserve;
            return reserve;
        }
    }

    pub fn edges(&self, edge_handle: EdgeHandle) -> Result< &[EdgeHandle], Error> {
        let (_, data) = Header::parse(&self.edges, edge_handle);
        return Ok(data);
    }

    pub fn edges_mut(&mut self, edge_handle: EdgeHandle) -> Result< &mut [EdgeHandle], Error>{
        let (_, data) = Header::parse_mut(&mut self.edges, edge_handle);
        return Ok(data);
    }
    #[cfg_attr(release, inline(always))]
    pub fn vertex_handle(&self, edge_handle: EdgeHandle) -> VertexHandle {
        return vertex_handle(&self.edges, edge_handle);
    }

    #[cfg_attr(release, inline(always))]
    pub fn handle_from_edge_handle(&self, edge_handle: EdgeHandle) -> Handle {
        return handle_from_edge_handle(&self.edges, edge_handle);
    }

    #[cfg_attr(release, inline(always))]
    fn inc_visited_flag(&mut self, edge_handle: EdgeHandle) {
        let header = Header::parse_mut(&mut self.edges, edge_handle).0;
        header.visited_flag += 1;
    }
    #[cfg_attr(release, inline(always))]
    fn set_visited_flag(&mut self, edge_handle: EdgeHandle, val: MSize) {
        let header = Header::parse_mut(&mut self.edges, edge_handle).0;
        header.visited_flag = val;
    }
    #[cfg_attr(release, inline(always))]
    fn visited_flag(&self, edge_handle: EdgeHandle) -> MSize {
        let header = Header::parse(&self.edges, edge_handle).0;
        return header.visited_flag;
    }

}


#[cfg_attr(release, inline(always))]
pub fn header_size_in_msize_units() -> usize {
    let raw_size = size_of::<Header>();
    ((raw_size + MSIZE_ALIGN_MASK) & !MSIZE_ALIGN_MASK) / size_of::<MSize>()
}


// pub fn dfs<T>(root: &Tree<T>, traverse: fn(node: &Tree<T>)){
//     let mut stack: Vec<(&Tree<T>, Iter<*mut Tree<T>>)> = Vec::new();
//     stack.push( (root, root.children.iter()));
//
//     while !stack.is_empty() {
//         let current_node = stack.last_mut().unwrap();
//
//         let child_node = current_node.1.next();
//         let parent_node = current_node.0;
//         match child_node {
//             None => {
//                 stack.pop();
//                 traverse(parent_node);
//             },
//             Some(child_node) => {
//                 stack.push( (child_node, child_node.children.iter()) );
//             }
//         }
//     }
// }

