#![allow(dead_code)]

#[derive(Debug, PartialEq, Eq)]
enum Type {
    Int,
    Pair,
}

#[derive(Debug)]
enum Value {
    Int(i64),
    Pair(Option<*mut Object>, Option<*mut Object>),
}

#[derive(Debug)]
struct Object {
    marked: bool,
    next: Option<*mut Object>,
    value: Value,
}

impl Object {
    fn mark(&mut self) {
        if self.marked {
            return;
        }
        self.marked = true;

        if let Value::Pair(head, tail) = self.value {
            if let Some(obj) = head {
                unsafe { (*obj).mark() }
            }
            if let Some(obj) = tail {
                unsafe { (*obj).mark() }
            }
        }
    }
}

struct VM {
    stack: Vec<*mut Object>,

    first_object: Option<*mut Object>,
    max_objects: usize,
    num_objects: usize,
}

impl VM {
    const STACK_MAX: usize = 256;
    const INITIAL_GC_THRESHOLD: usize = 32;
    fn new() -> VM {
        VM {
            stack: Vec::new(),
            first_object: None,
            max_objects: VM::INITIAL_GC_THRESHOLD,
            num_objects: 0,
        }
    }
    fn mark_all(&mut self) {
        for obj in &self.stack {
            unsafe { (**obj).mark() };
        }
    }

    fn sweep(&mut self) {
        // starting with a raw translation of the C
        unsafe {
            // Not going to lie -- double pointers are still mind bending.
            let mut object = &mut self.first_object;
            while object.is_some() {
                if !(*object.unwrap()).marked {
                    let unreached = *object;
                    *object = (*unreached.unwrap()).next;
                    self.num_objects -= 1;
                    drop(Box::from_raw(unreached.unwrap()));
                } else {
                    (*object.unwrap()).marked = false;
                    object = &mut (*object.unwrap()).next;
                }
            }
        }
    }

    fn gc(&mut self) {
        self.mark_all();
        self.sweep();
        self.max_objects = self.num_objects * 2;
    }

    fn push(&mut self, value: *mut Object) {
        self.stack.push(value);      
    }
    fn pop(&mut self) -> *mut Object {

        self.stack.pop().expect("Stack underflow!")
    }
    fn new_object(&mut self) -> *mut Object {
        if self.num_objects == self.max_objects {
            self.gc();
        }

        self.num_objects += 1;
        let value: Value = unsafe { ::std::mem::uninitialized() };

        let obj = Box::new(Object {
            marked: false,
            next: self.first_object,
            value,
        });

        let ptr = Box::into_raw(obj);
        self.first_object = Some(ptr);
        ptr
    }

    fn push_int(&mut self, value: i64) {
        let obj = self.new_object();
        unsafe {
            (*obj).value = Value::Int(value);
        };
        self.push(obj);
    }

    fn push_pair(&mut self) -> *mut Object {
        let obj = self.new_object();
        unsafe {
            (*obj).value = Value::Pair(Some(self.pop()), Some(self.pop()));
        }
        self.push(obj);
        obj
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stack_is_preserved() {
        let mut vm = VM::new();
        vm.push_int(1);
        vm.push_int(2);

        assert_eq!(vm.num_objects, 2, "Objects should be preserved");
    }
    #[test]
    fn collects_garbage() {
        let mut vm = VM::new();
        vm.push_int(1);
        vm.push_int(2);
        vm.pop();
        vm.pop();

        vm.gc();
        assert_eq!(vm.num_objects, 0, "Garbage should have been collected");
    }
    #[test]
    fn reach_nested() {
        let mut vm = VM::new();
        vm.push_int(1);
        vm.push_int(2);
        vm.push_pair();
        vm.push_int(3);
        vm.push_int(4);
        vm.push_pair();
        vm.push_pair();

        vm.gc();
        assert_eq!(vm.num_objects, 7, "Garbage should have been collected");
    }
    #[test]
    fn cycles() {
        let mut vm = VM::new();
        vm.push_int(1);
        vm.push_int(2);
        let a = vm.push_pair();

        vm.push_int(3);
        vm.push_int(4);
        let b = vm.push_pair();

        // Set up a cycle, and also make 2 and 4 unreachable and collectible

        unsafe {
            if let Value::Pair(head, _) = (*a).value {
                (*a).value = Value::Pair(head, Some(b));
            }
            if let Value::Pair(head, _) = (*b).value {
                (*b).value = Value::Pair(head, Some(a));
            }
        }

        vm.gc();
        assert_eq!(vm.num_objects, 4, "Should have collected cycles");
    }
    #[test]
    fn perf_test() {
        let mut vm = VM::new();

        for i in 0..1000 {
            for _ in 0..20 {
                vm.push_int(i);
            }
            for _ in 0..20 {
                vm.pop();
            }
        }
    }
}
