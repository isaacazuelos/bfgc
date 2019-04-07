#![allow(dead_code)]

#[derive(PartialEq, Eq)]
enum Type {
    Int,
    Pair,
}

struct Object {
    marked: bool,
    next: Option<*mut Object>,
    tag: Type,
    payload: Payload,
}

impl Object {
    fn mark(&mut self) {
        if self.marked {
            return;
        }
        self.marked = true;

        if self.tag == Type::Pair {
            let (head, tail) = unsafe { self.payload.pair };
            if let Some(obj) = head {
                unsafe { (*obj).mark() }
            }
            if let Some(obj) = tail {
                unsafe { (*obj).mark() }
            }
        }
    }
}

union Payload {
    int: i64,
    pair: (Option<*mut Object>, Option<*mut Object>),
}

struct VM {
    stack: [Option<*mut Object>; VM::STACK_MAX],
    stack_size: usize,
    first_object: Option<*mut Object>,
    max_objects: usize,
    num_objects: usize,
}

impl VM {
    const STACK_MAX: usize = 256;
    const INITIAL_GC_THRESHOLD: usize = 32;
    fn new() -> VM {
        VM {
            stack: [None; VM::STACK_MAX],
            stack_size: 0,
            first_object: None,
            max_objects: VM::INITIAL_GC_THRESHOLD,
            num_objects: 0,
        }
    }
    fn mark_all(&mut self) {
        for i in 0..self.stack_size {
            if let Some(obj) = self.stack[i] {
                unsafe {
                    (*obj).mark();
                };
            }
        }
    }
    fn sweep(&mut self) {
        let mut cursor = self.first_object;
        
        while let Some(object) = cursor {
            unsafe {
                if !(*object).marked {
                    let unreachable: *mut Object = object;
                    cursor = (*unreachable).next;
                    Box::from_raw(unreachable);
                    self.num_objects -= 1;
                } else {
                    (*object).marked = false;
                    cursor = (*object).next;
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
        assert!(self.stack_size < VM::STACK_MAX, "Stack overflow!");
        self.stack[self.stack_size] = Some(value);
        self.stack_size += 1;
    }
    fn pop(&mut self) -> *mut Object {
        assert!(self.stack_size > 0, "Stack underflow");
        self.stack_size -= 1;
        let obj = self.stack[self.stack_size];
        obj.unwrap()
    }

    fn new_object(&mut self, object_type: Type) -> *mut Object {
        if self.num_objects == self.max_objects {
            self.gc();
        }

        self.num_objects += 1;

        let payload = match object_type {
            Type::Int => Payload { int: 0 },
            Type::Pair => Payload { pair: (None, None) },
        };
        let obj = Box::new(Object {
            marked: false,
            next: self.first_object,
            tag: object_type,
            payload,
        });
        let ptr = Box::into_raw(obj);

        self.first_object = Some(ptr);
        ptr
    }

    fn push_int(&mut self, value: i64) {
        let obj = VM::new_object(self, Type::Int);
        unsafe {
            (*obj).payload = Payload { int: value };
        };
        self.push(obj);
    }

    fn push_pair(&mut self) -> *mut Object {
        let obj = VM::new_object(self, Type::Pair);
        unsafe {
            (*obj).payload = Payload {
                pair: (Some(self.pop()), Some(self.pop())),
            };
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
            (*a).payload.pair.1 = Some(b);
            (*b).payload.pair.1 = Some(a);
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
