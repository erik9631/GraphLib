use std::collections::HashMap;
use crate::graph;



#[test]
pub fn tree_view_create_child_test(){
    let mut graph = graph::Graph::new_large();
    let mut tree_view = graph.tree_view();

    let root = tree_view.create_node("root");
    let child1 = tree_view.create_child(root.edge_handle, "child1");
    let child2 = tree_view.create_child(root.edge_handle, "child2");
    let child3 = tree_view.create_child(root.edge_handle, "child3");
    let child1_1 = tree_view.create_child(child1.edge_handle, "child1_1");
    let child1_2 = tree_view.create_child(child1.edge_handle, "child1_2");

    let child1_2_1 = tree_view.create_child(child1_2.edge_handle, "child1_2_1");

    assert_eq!(tree_view.values[tree_view.vertex_handle(root.edge_handle)], "root");

    assert_eq!(tree_view.values[tree_view.vertex_handle(child1.edge_handle)], "child1");
    assert_eq!(tree_view.values[tree_view.get_parent(child1.edge_handle).vertex_handle], "root");
    assert_eq!(tree_view.values[tree_view.get_root(child1.edge_handle).vertex_handle], "root");

    assert_eq!(tree_view.values[tree_view.vertex_handle(child2.edge_handle)], "child2");
    assert_eq!(tree_view.values[tree_view.get_parent(child2.edge_handle).vertex_handle], "root");
    assert_eq!(tree_view.values[tree_view.get_root(child2.edge_handle).vertex_handle], "root");

    assert_eq!(tree_view.values[tree_view.vertex_handle(child3.edge_handle)], "child3");
    assert_eq!(tree_view.values[tree_view.get_parent(child3.edge_handle).vertex_handle], "root");
    assert_eq!(tree_view.values[tree_view.get_root(child3.edge_handle).vertex_handle], "root");

    assert_eq!(tree_view.values[tree_view.vertex_handle(child1_1.edge_handle)], "child1_1");
    assert_eq!(tree_view.values[tree_view.get_parent(child1_1.edge_handle).vertex_handle], "child1");
    assert_eq!(tree_view.values[tree_view.get_root(child1_1.edge_handle).vertex_handle], "root");

    assert_eq!(tree_view.values[tree_view.vertex_handle(child1_2.edge_handle)], "child1_2");
    assert_eq!(tree_view.values[tree_view.get_parent(child1_2.edge_handle).vertex_handle], "child1");
    assert_eq!(tree_view.values[tree_view.get_root(child1_2.edge_handle).vertex_handle], "root");

    assert_eq!(tree_view.values[tree_view.vertex_handle(child1_2_1.edge_handle)], "child1_2_1");
    assert_eq!(tree_view.values[tree_view.get_parent(child1_2_1.edge_handle).vertex_handle], "child1_2");
    assert_eq!(tree_view.values[tree_view.get_root(child1_2_1.edge_handle).vertex_handle], "root");
}

#[test]
pub fn tree_view_add_child_test() {
    let mut graph = graph::Graph::new_large();
    let mut tree_view = graph.tree_view();

    let root = tree_view.create_node("root");
    let child1 = tree_view.create_node("child1");
    let child2 = tree_view.create_node( "child2");
    let child3 = tree_view.create_node("child3");

    let child1_1 = tree_view.create_node("child1_1");
    let child1_2 = tree_view.create_node("child1_2");

    tree_view.add_child(root.edge_handle, child1.edge_handle);
    tree_view.add_child(root.edge_handle, child2.edge_handle);
    tree_view.add_child(root.edge_handle, child3.edge_handle);

    let children = tree_view.get_children(root.edge_handle);
    assert_eq!(children.len(), 3);

    tree_view.add_child(child1.edge_handle, child1_1.edge_handle);
    tree_view.add_child(child1.edge_handle, child1_2.edge_handle);

    let children = tree_view.get_children(child1.edge_handle);
    assert_eq!(children.len(), 2);

    assert_eq!(tree_view.values[tree_view.vertex_handle(root.edge_handle)], "root");

    assert_eq!(tree_view.values[tree_view.vertex_handle(child1.edge_handle)], "child1");
    assert_eq!(tree_view.values[tree_view.get_parent(child1.edge_handle).vertex_handle], "root");
    assert_eq!(tree_view.values[tree_view.get_root(child1.edge_handle).vertex_handle], "root");

    assert_eq!(tree_view.values[tree_view.vertex_handle(child2.edge_handle)], "child2");
    assert_eq!(tree_view.values[tree_view.get_parent(child2.edge_handle).vertex_handle], "root");
    assert_eq!(tree_view.values[tree_view.get_root(child2.edge_handle).vertex_handle], "root");

    assert_eq!(tree_view.values[tree_view.vertex_handle(child3.edge_handle)], "child3");
    assert_eq!(tree_view.values[tree_view.get_parent(child3.edge_handle).vertex_handle], "root");
    assert_eq!(tree_view.values[tree_view.get_root(child3.edge_handle).vertex_handle], "root");

    assert_eq!(tree_view.values[tree_view.vertex_handle(child1_1.edge_handle)], "child1_1");
    assert_eq!(tree_view.values[tree_view.get_parent(child1_1.edge_handle).vertex_handle], "child1");
    assert_eq!(tree_view.values[tree_view.get_root(child1_1.edge_handle).vertex_handle], "root");

    assert_eq!(tree_view.values[tree_view.vertex_handle(child1_2.edge_handle)], "child1_2");
    assert_eq!(tree_view.values[tree_view.get_parent(child1_2.edge_handle).vertex_handle], "child1");
    assert_eq!(tree_view.values[tree_view.get_root(child1_2.edge_handle).vertex_handle], "root");
}
#[test]
pub fn get_children_test(){
    let mut graph = graph::Graph::new_large();
    let mut tree_view = graph.tree_view();

    let root = tree_view.create_node("root");
    tree_view.create_child(root.edge_handle, "child1");
    tree_view.create_child(root.edge_handle, "child2");
    tree_view.create_child(root.edge_handle, "child3");

    let children = tree_view.get_children(root.edge_handle);

    let mut snap = HashMap::from({
        [
            ("child1", 0),
            ("child2", 1),
            ("child3", 2),
        ]
    });

    for child in children {
        let vertex_handle  = tree_view.vertex_handle(*child);
        let val = tree_view.values[vertex_handle];
        assert!(snap.contains_key(val));
        snap.remove(val);
    }
}
