use std::cmp::min;
use std::collections::HashMap;
use std::mem::size_of;
use std::time::{Instant};
use crate::{graph};
use crate::graph::{Graph, header_size_in_msize_units, MSize, TraverseResult};
use crate::traits::Transform;

#[test]
pub fn graph_init_test() {
    let mut graph = Graph::new_large();
    assert_eq!(graph.vertices.len(), 0);
    assert_eq!(graph.edges.capacity(), 0);

    graph.create_leaf(1);
    graph.create_leaf(2);
    graph.create_leaf(3);

    assert_eq!(graph.vertices.len(), 3);
    assert_eq!(graph.edges.capacity(), (50+ header_size_in_msize_units())*3);

}

#[test]
pub fn graph_basic_test(){
    let mut graph = Graph::new_large();
    let a = graph.create_leaf("a");
    let b = graph.create_leaf("b");
    graph.create_leaf("c");

    graph.create_and_connect_leaf(a.edge_handle, "a_a");
    graph.create_and_connect_leaf(a.edge_handle, "a_b");
    graph.create_and_connect_leaf(a.edge_handle, "a_c");

    let b_a = graph.create_and_connect_leaf(b.edge_handle, "b_a");
    graph.create_and_connect_leaf(b.edge_handle, "b_b");

   graph.create_and_connect_leaf(b_a.edge_handle, "b_a_a");

    let a_edges_result = graph.edges.edges(a.edge_handle);
    assert_eq!(a_edges_result.is_err(), false);

    let a_edges = a_edges_result.ok().unwrap();
    assert_eq!(a_edges.len(), 3);

    let mut snap1 = HashMap::from([
        ("a_a".to_string(), 1),
        ("a_b".to_string(), 2),
        ("a_c".to_string(), 3),
    ]);

    for edge in a_edges {
        let key = graph.vertices[graph.edges.vertex_handle(*edge)];
        assert!(snap1.contains_key(key));
        snap1.remove(key);
    }

    let b_edges_result = graph.edges.edges(b.edge_handle);
    assert_eq!(b_edges_result.is_err(), false);

    let b_edges = b_edges_result.ok().unwrap();
    assert_eq!(b_edges.len(), 2);

    let mut snap2 = HashMap::from([
        ("b_a".to_string(), 1),
        ("b_b".to_string(), 2),
    ]);

    for edge in b_edges {
        let key = graph.vertices[graph.edges.vertex_handle(*edge)];
        assert!(snap2.contains_key(key));
        snap2.remove(key);
    }

    let b_a_a_edges_result = graph.edges.edges(b_a.edge_handle);
    assert_eq!(b_a_a_edges_result.is_err(), false);

    let b_a_a_edges = b_a_a_edges_result.ok().unwrap();
    assert_eq!(b_a_a_edges.len(), 1);

    for edge in b_a_a_edges {
        assert_eq!(graph.vertices[graph.edges.vertex_handle(*edge)], "b_a_a");
    }

}

#[test]
pub fn graph_default_capacity_test(){
    let mut graph = Graph::new_large();
    let count = 50;


    for i in 0..count {
        graph.create_leaf(i);
    }

    assert_eq!(graph.vertices.len(), 50);
    assert_eq!(graph.edges.capacity(), (50+ header_size_in_msize_units())*count);
}

#[test]
pub fn graph_with_capacity_test(){
    let mut graph = graph::Graph::with_reserve(10);
    let count = 100;

    for i in 0..count {
        graph.create_leaf(i);
    }

    assert_eq!(graph.edges.capacity(), (10+ header_size_in_msize_units())*count);
}

#[test]
#[should_panic]
pub fn graph_edge_overflow_test(){
    let mut graph = graph::Graph::with_reserve(3);
    let count = 4;
    let a = graph.create_leaf(0);

    for i in 0..count {
        graph.create_and_connect_leaf(a.edge_handle, i+1);
    }
}


#[test]
pub fn graph_mutability_test(){
    let mut graph = graph::Graph::new_large();
    let a = graph.create_leaf("a");
    graph.create_leaf("b");
    graph.create_leaf("c");

    graph.create_and_connect_leaf(a.edge_handle, "a_a");
    graph.create_and_connect_leaf(a.edge_handle, "a_b");
    graph.create_and_connect_leaf(a.edge_handle, "a_c");

    let result = graph.edges.edges(a.edge_handle);
    assert_eq!(result.is_err(), false);

    let edges = result.ok().unwrap();
    assert_eq!(edges.len(), 3);

    let mut snap = HashMap::from([
        ("a_a".to_string(), "a_a_edited"),
        ("a_b".to_string(), "a_b_edited"),
        ("a_c".to_string(), "a_c_edited"),
    ]);

    for edge in edges {
        let handle = graph.edges.vertex_handle(*edge);
        let key = graph.vertices[handle];
        assert!(snap.contains_key(key));

    }
}

#[test]
pub fn graph_transform_bench(){
    let mut graph = Graph::new_large();
    let test_size = min(size_of::<MSize>(), 10000000) as MSize;

    for i in 0..test_size {
        graph.create_leaf(i);
    }
    let start = Instant::now();
    graph.vertices.transform(|slice| {
        for i in slice{
            *i = *i * 10;
        }
    });
    println!("Time taken: {:?}", start.elapsed());
    for i in 0..test_size {
        assert_eq!(*graph.vertices.at(i), i*10);
    }


}

#[test]
pub fn graph_transform_bench_async(){
    let mut graph = Graph::new_large();
    let test_size = min(size_of::<MSize>(), 10000000) as MSize;

    for i in 0..test_size {
        graph.create_leaf(i);
    }
    let start = Instant::now();
    graph.vertices.async_transform(|slice| {
        for i in slice{
            *i = *i * 10;
        }
    });
    println!("Time taken: {:?}", start.elapsed());

    for i in 0..test_size {
        assert_eq!(*graph.vertices.at(i), i*10);
    }


}
#[test]
pub fn graph_disconnect_test(){
    let mut graph = Graph::new_large();
    let a = graph.create_leaf("a");
    graph.create_leaf("b");
    graph.create_leaf("c");

    graph.create_and_connect_leaf(a.edge_handle, "a_a");
    let ab= graph.create_and_connect_leaf(a.edge_handle, "a_b");
    graph.create_and_connect_leaf(a.edge_handle, "a_c");
    let ad= graph.create_and_connect_leaf(a.edge_handle, "a_d");
    graph.create_and_connect_leaf(a.edge_handle, "a_e");
    let af= graph.create_and_connect_leaf(a.edge_handle, "a_f");
    graph.edges.disconnect(a.edge_handle, af.edge_handle);


    assert_eq!(graph.edges.len(a.edge_handle), 5);

    let edges = graph.edges.edges(a.edge_handle).unwrap();
    let mut expected = HashMap::from([
        ("a_a".to_string(), true),
        ("a_b".to_string(), true),
        ("a_c".to_string(), true),
        ("a_d".to_string(), true),
        ("a_e".to_string(), true),
    ]);

    for edge in edges {
        let handle = graph.edges.vertex_handle(*edge);
        let key = graph.vertices[handle].to_string();
        assert!(expected.contains_key(&key));
        expected.remove(&key);
    }

    graph.edges.disconnect(a.edge_handle, ad.edge_handle);
    graph.edges.disconnect(a.edge_handle, ab.edge_handle);

    assert_eq!(graph.edges.len(a.edge_handle), 3);

    let mut expected = HashMap::from([
        ("a_a".to_string(), true),
        ("a_c".to_string(), true),
        ("a_e".to_string(), true),
    ]);

    let edges = graph.edges.edges(a.edge_handle).unwrap();

    for edge in edges {
        let handle = graph.edges.vertex_handle(*edge);
        let key = graph.vertices[handle].to_string();
        assert!(expected.contains_key(&key));
        expected.remove(&key);
    }

}
#[test]
pub fn graph_bfs_test(){
    let mut graph = Graph::new_large();
    let root = graph.create_leaf("root");
    let a = graph.create_and_connect_leaf(root.edge_handle, "a");
    let b = graph.create_and_connect_leaf(root.edge_handle, "b");
    graph.create_and_connect_leaf(root.edge_handle, "c");

    graph.create_and_connect_leaf(a.edge_handle, "a_a");
    graph.create_and_connect_leaf(a.edge_handle, "a_b");
    graph.create_and_connect_leaf(a.edge_handle, "a_c");

    let b_a = graph.create_and_connect_leaf(b.edge_handle, "b_a");
    graph.create_and_connect_leaf(b.edge_handle, "b_b");

    graph.create_and_connect_leaf(b_a.edge_handle, "b_a_a");

    // Instead of traverse, it should just save them to a memory and return the content to you. Faster than function calls and u can do iteration on your own.

    let mut expected = HashMap::from([
        ("root".to_string(), true),
        ("a_c".to_string(), true),
        ("a_e".to_string(), true),
    ]);

    let mut snap = vec![
        "b_a_a",
        "b_b",
        "b_a",
        "a_c",
        "a_b",
        "a_a",
        "c",
        "b",
        "a",
        "root",
    ];


    graph.bfs(root.edge_handle, move |graph, vertex| {
        if snap.pop().unwrap() != graph.vertices[vertex.vertex_handle]{
            assert!(false);
        }

        return TraverseResult::Continue;
    });
}

#[test]
pub fn graph_static_test(){
    let mut graph = Graph::new();
    let root = graph.create("root", 5);
    let a = graph.create_and_connect(root.edge_handle,"a", 1);
    assert_eq!(graph.edges.reserve(root.edge_handle), 5);
    let b = graph.create_and_connect(root.edge_handle, "b", 0);
    assert_eq!(graph.edges.reserve(root.edge_handle), 5);
    let c = graph.create_and_connect(root.edge_handle,"c", 0);
    assert_eq!(graph.edges.reserve(root.edge_handle), 5);
    let d = graph.create_and_connect(root.edge_handle, "d", 1);
    assert_eq!(graph.edges.reserve(root.edge_handle), 5);
    let e = graph.create_and_connect(root.edge_handle, "e", 1);
    assert_eq!(graph.edges.reserve(root.edge_handle), 5);

    graph.create_and_connect(a.edge_handle, "a_a", 0);
    assert_eq!(graph.edges.reserve(root.edge_handle), 5);
    graph.create_and_connect(d.edge_handle, "a_d", 0);
    assert_eq!(graph.edges.reserve(root.edge_handle), 5);
    graph.create_and_connect(e.edge_handle, "a_e", 0);
    assert_eq!(graph.edges.reserve(root.edge_handle), 5);
    assert_eq!(graph.edges.reserve(root.edge_handle), 5);
    // for vertex in graph.bfs(root){
    //     match graph.vertices[vertex]{
    //         "root" => {
    //             assert_eq!(graph.edges.len(vertex), 5);
    //             assert_eq!(graph.edges.reserve(vertex), 5);
    //         },
    //         "a" => {
    //             assert_eq!(graph.edges.len(vertex), 1);
    //             assert_eq!(graph.edges.reserve(vertex), 1);
    //         },
    //         "b" => {
    //             assert_eq!(graph.edges.len(vertex), 0);
    //             assert_eq!(graph.edges.reserve(vertex), 0);
    //         },
    //         "c" => {
    //             assert_eq!(graph.edges.len(vertex), 0);
    //             assert_eq!(graph.edges.reserve(vertex), 0);
    //         },
    //         "d" => {
    //             assert_eq!(graph.edges.len(vertex), 1);
    //             assert_eq!(graph.edges.reserve(vertex), 1);
    //         },
    //         "e" => {
    //             assert_eq!(graph.edges.len(vertex), 1);
    //             assert_eq!(graph.edges.reserve(vertex), 1);
    //         },
    //         "a_a" => {
    //             assert_eq!(graph.edges.len(vertex), 0);
    //             assert_eq!(graph.edges.reserve(vertex), 0);
    //         },
    //         "a_d" => {
    //             assert_eq!(graph.edges.len(vertex), 0);
    //             assert_eq!(graph.edges.reserve(vertex), 0);
    //         },
    //         "a_e" => {
    //             assert_eq!(graph.edges.len(vertex), 0);
    //             assert_eq!(graph.edges.reserve(vertex), 0);
    //         },
    //         _ => continue,
    //     }
    // }

}