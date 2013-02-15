//
// The core DOM types. Defines the basic DOM hierarchy as well as all the HTML elements.
//

use dom::bindings;
use dom::document::Document;
use dom::element::{Element, ElementTypeId, HTMLImageElement, HTMLImageElementTypeId};
use dom::element::{HTMLStyleElementTypeId};
use dom::window::Window;
use layout::debug::DebugMethods;
use layout::flow::FlowContext;
use newcss::complete::CompleteSelectResults;

use core::cast::transmute;
use core::ptr::null;
use geom::size::Size2D;
use js::crust::*;
use js::rust::Compartment;
use std::arc::ARC;

//
// The basic Node structure
//

/// This is what a Node looks like if you do not know what kind of node it is. To unpack it, use
/// downcast().
///
/// FIXME: This should be replaced with a trait once they can inherit from structs.
pub struct AbstractNode {
    priv obj: *mut Node
}

impl Eq for AbstractNode {
    pure fn eq(&self, other: &AbstractNode) -> bool { self.obj == other.obj }
    pure fn ne(&self, other: &AbstractNode) -> bool { self.obj != other.obj }
}

pub struct Node {
    type_id: NodeTypeId,

    parent_node: Option<AbstractNode>,
    first_child: Option<AbstractNode>,
    last_child: Option<AbstractNode>,
    next_sibling: Option<AbstractNode>,
    prev_sibling: Option<AbstractNode>,

    // You must not touch this if you are not layout.
    priv layout_data: Option<@LayoutData>
}

#[deriving_eq]
pub enum NodeTypeId {
    DoctypeNodeTypeId,
    CommentNodeTypeId,
    ElementNodeTypeId(ElementTypeId),
    TextNodeTypeId,
}

//
// Auxiliary layout data
//

pub struct LayoutData {
    style: Option<CompleteSelectResults>,
    flow: Option<@FlowContext>,
}

impl LayoutData {
    static pub fn new() -> LayoutData {
        LayoutData {
            style: None,
            flow: None,
        }
    }
}

//
// Basic node types
//

pub struct Doctype {
    parent: Node,
    name: ~str,
    public_id: Option<~str>,
    system_id: Option<~str>,
    force_quirks: bool
}

impl Doctype {
    static pub fn new(name: ~str,
                      public_id: Option<~str>,
                      system_id: Option<~str>,
                      force_quirks: bool)
                   -> Doctype {
        Doctype {
            parent: Node::new(DoctypeNodeTypeId),
            name: name,
            public_id: public_id,
            system_id: system_id,
            force_quirks: force_quirks,
        }
    }
}

pub struct Comment {
    parent: Node,
    text: ~str,
}

impl Comment {
    static pub fn new(text: ~str) -> Comment {
        Comment {
            parent: Node::new(CommentNodeTypeId),
            text: text
        }
    }
}

pub struct Text {
    parent: Node,
    text: ~str,
}

impl Text {
    static pub fn new(text: ~str) -> Text {
        Text {
            parent: Node::new(CommentNodeTypeId),
            text: text
        }
    }
}

impl AbstractNode {
    fn as_node(&self) -> &self/mut Node {
        transmute(self.obj)
    }

    //
    // Convenience accessors
    //
    // FIXME: Fold these into util::tree.

    fn type_id(self)      -> NodeTypeId           { self.as_node().type_id      }
    fn parent(self)       -> Option<AbstractNode> { self.as_node().parent       }
    fn first_child(self)  -> Option<AbstractNode> { self.as_node().first_child  }
    fn last_child(self)   -> Option<AbstractNode> { self.as_node().last_child   }
    fn prev_sibling(self) -> Option<AbstractNode> { self.as_node().prev_sibling }
    fn next_sibling(self) -> Option<AbstractNode> { self.as_node().next_sibling }

    //
    // Tree operations
    //
    // FIXME: Fold this into util::tree.
    //

    fn is_leaf(self) -> bool { self.first_child().is_none() }

    // Invariant: `child` is disconnected from the document.
    fn append_child(self, child: AbstractNode) {
        assert self != child;

        let parent_n = self.as_node();
        let child_n = child.as_node();

        assert child_n.parent.is_none();
        assert child_n.prev_sibling.is_none();
        assert child_n.next_sibling.is_none();

        child_n.parent = self;

        match parent_n.last_child {
            None => parent_n.first_child = Some(child),
            Some(last_child) => {
                let last_child_n = last_child.as_node();
                assert last_child_n.next_sibling.is_none();
                last_child_n.next_sibling = Some(child);
            }
        }

        child_n.prev_sibling = parent_n.last_child;
    }

    //
    // Tree traversal
    //
    // FIXME: Fold this into util::tree.
    //

    fn each_child(self, f: &fn(AbstractNode) -> bool) {
        let current_opt = self.as_node().first_child;
        while !current_opt.is_none() {
            let current = current_opt.get();
            if !f(current) {
                break;
            }
            current_opt = current.next_sibling();
        }
    }

    fn traverse_preorder(self, f: &fn(AbstractNode) -> bool) -> bool {
        if !f(self) {
            return false;
        }
        for self.each_child |kid| {
            if !self.traverse_preorder(kid) {
                return false;
            }
        }
        true
    }

    fn traverse_postorder(self, f: &fn(AbstractNode) -> bool) -> bool {
        for self.each_child |kid| {
            if !self.traverse_postorder(kid) {
                return false;
            }
        }
        f(self)
    }

    //
    // Downcasts
    //

    fn is_text(self) -> bool { self.as_node().type_id == TextNodeTypeId }

    fn as_text(&self) -> &self/mut Text {
        if !self.is_text() {
            fail!(~"node is not text");
        }
        unsafe {
            transmute(self.obj)
        }
    }

    fn is_element(self) -> bool {
        match self.as_node().type_id {
            ElementNodeTypeId(*) => true,
            _ => false
        }
    }

    fn as_element(&self) -> &self/mut Element {
        if !self.is_element() {
            fail!(~"node is not an element");
        }
        unsafe {
            transmute(self.obj)
        }
    }

    fn is_image_element(self) -> bool {
        self.as_node().type_id == ElementNodeTypeId(HTMLImageElementTypeId)
    }

    fn as_image_element(&self) -> &self/mut HTMLImageElement {
        if !self.is_image_element() {
            fail!(~"node is not an image element");
        }
        unsafe {
            transmute(self.obj)
        }
    }

    fn is_style_element(self) -> bool {
        self.as_node().type_id == ElementNodeTypeId(HTMLStyleElementTypeId)
    }
}

impl Node {
    static pub unsafe fn as_abstract_node<N>(node: ~N) -> AbstractNode {
        // This surrenders memory management of the node!
        AbstractNode {
            obj: transmute(node)
        }
    }

    static pub fn new(type_id: NodeTypeId) -> Node {
        Node {
            type_id: type_id,

            parent_node: None,
            first_child: None,
            last_child: None,
            next_sibling: None,
            prev_sibling: None,

            layout_data: None,
        }
    }
}

impl DebugMethods for Node {
    /* Dumps the subtree rooted at this node, for debugging. */
    pure fn dump(&self) {
        self.dump_indent(0u);
    }

    /* Dumps the node tree, for debugging, with indentation. */
    pure fn dump_indent(&self, indent: uint) {
        let mut s = ~"";
        for uint::range(0u, indent) |_i| {
            s += ~"    ";
        }

        s += self.debug_str();
        debug!("%s", s);

        // FIXME: this should have a pure version?
        unsafe {
            for self.each_child() |kid| {
                kid.dump_indent(indent + 1u) 
            }
        }
    }

    pure fn debug_str(&self) -> ~str {
        fmt!("%?", self.type_id)
    }
}

pub fn define_bindings(compartment: @mut Compartment, doc: @Document, win: @Window) {
    bindings::window::init(compartment, win);
    bindings::document::init(compartment, doc);
    bindings::node::init(compartment);
    bindings::element::init(compartment);
}

