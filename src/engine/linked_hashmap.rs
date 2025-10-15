use std::{hash::Hash};
use std::collections::{HashMap};

/* 
    Types implementing `HasId` must define an associated `Id` type
    that is `Eq`, `Hash`, and `Clone`, making it suitable for use
    as a key in hash-based collections.
*/
pub trait HasId {
    type Id: Eq + Hash + Clone;
    fn id(&self) -> Self::Id;
}

/* 
    A doubly linked list node holding a value that implements `HasId`.
    Each `Node` stores the element (`value`) and optional identifiers
    of its previous and next nodes (`prev`, `next`).
    The node’s own ID is derived from its inner value.
*/
#[derive(Debug)]
struct Node<T: HasId> {
    pub value: T,
    pub next: Option<T::Id>,
    pub prev: Option<T::Id>,
}
impl<T: HasId> Node<T> {
    pub fn new(value: T, prev: Option<T::Id>, next: Option<T::Id>) -> Self {
        Self { value, prev, next }
    }

    pub fn id(&self) -> T::Id {
        self.value.id()
    }
}
impl<T: HasId> HasId for Node<T> {
    type Id = T::Id;

    fn id(&self) -> Self::Id {
        self.value.id()
    }
}

/* 
    A hash-based doubly linked list that preserves FIFO order while
    allowing O(1) access, insertion, and removal by key.

    Each element is stored in a `Node` that tracks links to its previous
    and next nodes using their IDs.
    - `head` and `tail` keep track of the list boundaries.  
    - `items` is a `HashMap` for fast lookup by ID.
*/
#[derive(Debug)]
pub struct LinkedHashmap<T: HasId> {
    head: Option<T::Id>,
    tail: Option<T::Id>,
    items: HashMap<T::Id, Node<T>>,
}
impl<T> LinkedHashmap<T>
where
    T: HasId,
    T::Id: Eq + Hash + Clone,
{
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
            items: HashMap::new(),
        }
    }

    pub fn push(&mut self, value: T) {
        let id = value.id();
        let mut new_node = Node::new(value, None, None);

        if self.items.contains_key(&id) {
            // Do nothing if element's id already exists
            return;
        }

        if let Some(ref tail_id) = self.tail {
            if let Some(tail_node) = self.items.get_mut(tail_id) {
                tail_node.next = Some(id.clone());
                new_node.prev = Some(tail_node.id());
            }
        } else {
            self.head = Some(id.clone());
        }

        self.tail = Some(id.clone());
        self.items.insert(id, new_node);
    }

    pub fn push_first(&mut self, value: T) {
        let id = value.id();
        let mut new_node = Node::new(value, None, None);

        if self.items.contains_key(&id) {
            // Do nothing if element's id already exists
            return;
        }

        if let Some(ref head_id) = self.head {
            if let Some(head_node) = self.items.get_mut(head_id) {
                head_node.prev = Some(id.clone());
                new_node.next = Some(head_node.id());
            }
        } else {
            self.tail = Some(id.clone());
        }

        self.head = Some(id.clone());
        self.items.insert(id, new_node);
    }

    pub fn pop(&mut self) -> Option<T> {
        let head_id = self.head.take()?;
        
        let node = match self.items.remove(&head_id) {
            Some(n) => n,
            None => {
                self.head = None;
                self.tail = None;
                return None;
            }
        };

        self.head = node.next.clone();

        if let Some(ref new_head_id) = self.head {
            if let Some(new_head_node) = self.items.get_mut(new_head_id) {
                new_head_node.prev = None;
            } else {
                self.head = None;
                self.tail = None;
            }
        } else {
            self.tail = None;
        }

        Some(node.value)
    }

    pub fn remove(&mut self, id: &T::Id) -> Option<T> {
        let node = match self.items.remove(id) {
            Some(n) => n,
            None => return None,
        };

        if let Some(next_id) = node.next.clone() {
            if let Some(next_node) = self.items.get_mut(&next_id) {
                next_node.prev = node.prev.clone();
            } else if self.tail == Some(next_id) {
                self.tail = node.prev.clone();
            }
        } else {
            self.tail = node.prev.clone();
        }

        if let Some(prev_id) = node.prev.clone() {
            if let Some(prev_node) = self.items.get_mut(&prev_id) {
                prev_node.next = node.next.clone();
            } else if self.head == Some(prev_id) {
                self.head = node.next.clone();
            }
        } else {
            self.head = node.next.clone();
        }

        Some(node.value)
    }

    pub fn peek(&self) -> Option<&T> {
        self.head
            .as_ref()
            .and_then(|id| self.items.get(id))
            .map(|node| &node.value)
    }

    pub fn peek_tail(&self) -> Option<&T> {
        self.tail
            .as_ref()
            .and_then(|id| self.items.get(id))
            .map(|node| &node.value)
    }

    pub fn get(&self, id: &T::Id) -> Option<&T> {
        self.items.get(id).map(|node| &node.value)
    }

    pub fn get_mut(&mut self, id: &T::Id) -> Option<&mut T> {
        self.items.get_mut(id).map(|node| &mut node.value)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn contains(&self, id: &T::Id) -> bool {
        self.items.contains_key(id)
    }

    pub fn clear(&mut self) {
        self.head = None;
        self.tail = None;
        self.items.clear();
    }
}
impl<T: HasId> Default for LinkedHashmap<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[derive(Debug)]
    struct Item {
        id: Uuid,
    }
    impl Item {
        pub fn new(id: Uuid) -> Self {
            Self {
                id,
            }
        }
    }
    impl HasId for Item {
        type Id = Uuid;

        fn id(&self) -> Self::Id {
            self.id
        }
    }

    #[test]
    fn creation() {
        let linked: LinkedHashmap<Item> = LinkedHashmap::new();
        assert!(linked.is_empty());
    }

    #[test]
    fn push_pop_elements() {
        let mut linked: LinkedHashmap<Item> = LinkedHashmap::new();
        let item1 = Item::new(Uuid::new_v4());
        let item1_id = item1.id.clone();
        let item2 = Item::new(Uuid::new_v4());

        linked.push(item1);
        linked.push(item2);

        assert_eq!(linked.is_empty(), false);
        assert!(linked.contains(&item1_id));
        assert_eq!(linked.len(), 2);

        let pop_item = linked.pop();

        assert_eq!(&pop_item.unwrap().id, &item1_id);

        linked.pop();
        
        assert_eq!(linked.len(), 0);
        assert!(linked.is_empty());
    }

    #[test]
    fn remove_elements() {
        let mut linked: LinkedHashmap<Item> = LinkedHashmap::new();
        let item1 = Item::new(Uuid::new_v4());
        let item1_id = item1.id.clone();

        linked.push(item1);

        assert_eq!(linked.is_empty(), false);
        assert!(linked.contains(&item1_id));
        assert_eq!(linked.len(), 1);

        let removed_item = linked.remove(&item1_id);

        assert!(linked.is_empty());
        assert_eq!(linked.contains(&removed_item.unwrap().id), false);
        assert_eq!(linked.len(), 0);
    }

    #[test]
    fn peek_and_peek_tail() {
        let mut linked: LinkedHashmap<Item> = LinkedHashmap::new();
        let tail = Item::new(Uuid::new_v4());
        let tail_id = tail.id;
        let head = Item::new(Uuid::new_v4());
        let head_id = head.id;

        linked.push(tail);
        linked.push(head);

        assert!(linked.peek().unwrap().id == tail_id);
        assert_eq!(linked.peek_tail().unwrap().id == tail_id, false);
        assert_eq!(linked.peek().unwrap().id == head_id, false);
        assert!(linked.peek_tail().unwrap().id == head_id);
    }

    #[test]
    fn clear() {
        let mut linked: LinkedHashmap<Item> = LinkedHashmap::new();
        let item1 = Item::new(Uuid::new_v4());
        let item1_id = item1.id.clone();

        linked.push(item1);

        assert_eq!(linked.is_empty(), false);
        assert!(linked.contains(&item1_id));
        assert_eq!(linked.len(), 1);

        linked.clear();

        assert!(linked.is_empty());
        assert_eq!(linked.len(), 0);
    }
}