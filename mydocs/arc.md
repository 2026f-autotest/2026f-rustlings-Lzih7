# Rust 中 Arc 详解

## 1. Arc 解决什么问题

`Arc<T>` 是支持跨线程共享所有权的智能指针，位于 `std::sync::Arc`。

Arc 是 Atomically Reference Counted 的缩写，即“原子引用计数”。

普通所有权模型中，一个值只有一个所有者。实际程序却可能需要多个线程共同使用同一份数据，并且无法提前确定哪个线程最后结束。`Arc` 通过引用计数管理这份数据：

- 创建 `Arc` 时，建立一份共享数据和一个强引用。
- 克隆 `Arc` 时，新增一个共享所有权的句柄，强引用计数加一。
- 一个句柄被销毁时，强引用计数减一。
- 最后一个强引用被销毁时，内部数据 `T` 被销毁。

如果仍有 `Weak` 弱引用，共享分配的管理空间暂时保留；等弱引用也全部释放后才完全回收。

注意：引用计数是运行时机制，不是生命周期参数那样只用于编译期检查的标注。

## 2. 结合 arc1.rs 理解

练习中的关键代码如下：

```rust
use std::sync::Arc;
use std::thread;

let numbers: Vec<u32> = (0..100).collect();
let shared_numbers = Arc::new(numbers);
let mut handles = Vec::new();

for offset in 0..8 {
    let child_numbers = Arc::clone(&shared_numbers);
    handles.push(thread::spawn(move || {
        let sum: u32 = child_numbers
            .iter()
            .filter(|&&n| n % 8 == offset)
            .sum();
        (offset, sum)
    }));
}

let mut total = 0;
for handle in handles {
    let (offset, sum) = handle.join().unwrap();
    println!("offset={offset}, sum={sum}");
    total += sum;
}
assert_eq!(total, 4950);
```

这里 8 个线程共享的是同一个 `Vec<u32>`，不是每个线程各自复制一个向量。

每个线程负责一组数：

| offset | 参与求和的数字 | 个数 | 和 |
| --- | --- | --- | --- |
| 0 | 0, 8, ..., 96 | 13 | 624 |
| 1 | 1, 9, ..., 97 | 13 | 637 |
| 2 | 2, 10, ..., 98 | 13 | 650 |
| 3 | 3, 11, ..., 99 | 13 | 663 |
| 4 | 4, 12, ..., 92 | 12 | 576 |
| 5 | 5, 13, ..., 93 | 12 | 588 |
| 6 | 6, 14, ..., 94 | 12 | 600 |
| 7 | 7, 15, ..., 95 | 12 | 612 |

例如第一组是等差数列，和为 `(0 + 96) * 13 / 2 = 624`。八组之和为 `4950`，也就是 `0 + 1 + ... + 99`。

原练习在各个子线程内部打印，所以输出顺序不固定。上面的示例改为按句柄顺序等待后打印，方便阅读结果。

## 3. Arc::new(numbers) 做了什么

```rust
use std::sync::Arc;

let numbers = vec![1, 2, 3];
let shared_numbers: Arc<Vec<i32>> = Arc::new(numbers);

// numbers 的所有权已经移动，之后通过 shared_numbers 访问。
assert_eq!(shared_numbers[0], 1);
assert_eq!(Arc::strong_count(&shared_numbers), 1);
```

`Arc::new(numbers)` 将 `numbers` 的所有权交给 `Arc`，不是借用 `numbers`。

这里有两层堆内存需要区分：

- `Arc` 的共享分配中保存引用计数和 `Vec` 值。
- `Vec` 值内部保存指向元素缓冲区的指针，以及长度和容量；元素缓冲区本身已经在堆上。

把已有的 `Vec` 移进 `Arc` 不会复制其所有元素。原元素缓冲区继续由这个 `Vec` 管理。

## 4. clone() 到底复制了什么

练习里的：

```text
let child_numbers = shared_numbers.clone();
```

对于这里的 `Arc<Vec<u32>>`，等价于：

```text
let child_numbers = Arc::clone(&shared_numbers);
```

推荐第二种写法，因为它明确表明“克隆的是 Arc 句柄”。

```rust
use std::sync::Arc;

let a = Arc::new(vec![1, 2, 3]);
let b = Arc::clone(&a);

assert!(Arc::ptr_eq(&a, &b));
assert_eq!(Arc::strong_count(&a), 2);

drop(b);
assert_eq!(Arc::strong_count(&a), 1);
```

结构可以理解为：

```text
a --------+
          +----> 共享分配：引用计数 + 同一个 Vec
b --------+                           |
                                      +----> 元素缓冲区 [1, 2, 3]
```

`Arc::clone` 不会递归调用内部 `T` 的 `clone()`，也不要求 `T: Clone`。它只创建另一个指向同一分配的句柄，并原子地增加强引用计数，通常是 O(1) 操作。

对比：

| 操作 | 含义 |
| --- | --- |
| `Arc::clone(&a)` 或 `a.clone()` | 克隆 `Arc` 句柄，共享原来的 `Vec` |
| `a.as_ref().clone()` | 对内部 `Vec` 执行克隆，得到独立的新向量 |
| `let b = a;` | 移动句柄，不增加强引用计数，之后不能再使用 `a` |

这里讨论的是 `Arc<Vec<_>>`。`Vec::clone()` 会逐个克隆元素，但不代表任意元素类型的克隆都是“深拷贝”，例如元素本身也可能是 `Arc`。

## 5. 为什么需要先 clone，再 move

关键逻辑是：

```text
let child_numbers = Arc::clone(&shared_numbers);
thread::spawn(move || {
    // 使用 child_numbers
});
```

两步分别负责不同的事：

1. `clone`：创建一个新的共享所有权句柄，主线程保留原句柄。
2. `move`：把新句柄的所有权转移到闭包，让子线程持有它。

`move` 不会再增加引用计数，也不会把整个向量复制一遍。

如果直接把 `shared_numbers` 移进第一个线程，主线程就失去了这个句柄，下一轮循环无法再使用它。因此需要每轮先克隆一个句柄。

`thread::spawn` 要求闭包及其返回值满足 `Send + 'static` 等约束。普通线程可能比创建它的函数活得更久，所以不能随意借用该函数的局部变量。

这里 `'static` 约束不意味着线程或 `Arc` 必须活到程序结束，而是不能携带有效期不够长的借用。`Arc<Vec<u32>>` 拥有数据，因此可以满足要求。

反例思路：`Arc<&'a str>` 仍然包含生命周期为 `'a` 的借用。把短期借用装进 `Arc`，不会自动把它变成 `'static`。

另一个选择是 `std::thread::scope`：作用域线程确保退出作用域前等待线程结束，因此在满足其他借用条件时可以直接借用局部数据，不必所有多线程场景都使用 `Arc`。

## 6. 引用计数如何变化

以这道练习为例：

```text
Arc::new(numbers)             强引用计数为 1
创建第一个子线程句柄          加 1
创建第二个子线程句柄          再加 1
...
某个子线程退出，句柄被销毁    减 1
主线程的句柄被销毁            再减 1
最后一个强引用消失            销毁 Vec，释放其元素缓冲区
```

如果 8 个子线程句柄同时存在，加上主线程句柄，计数就是 9。

但不能断言“循环刚结束时一定是 9”：子线程可能已经结束并释放了自己的句柄。

`Arc::strong_count` 适合观察和调试。在并发场景下，读到的只是一个随时可能变化的快照，不应靠“计数等于 1”自行判断能否安全修改数据。

`join()` 的作用是等待线程结束并获取结果；共享数据的生命周期则由所有强引用共同决定。即使主线程先释放自己的句柄，仍持有句柄的子线程也能继续使用数据。

## 7. 为什么能直接调用 iter()

`child_numbers` 的类型是 `Arc<Vec<u32>>`，但它可以直接调用：

```text
child_numbers.iter()
```

因为 `Arc<T>` 实现了 `Deref<Target = T>`，方法调用时 Rust 可以自动解引用到内部的 `Vec`，再通过相应的方法解析访问其元素。

这不表示拿走了 `Vec`，这里只是借用它来遍历。

顺便拆解：

```text
.filter(|&&n| n % 8 == offset)
```

- `iter()` 产生 `&u32`。
- `filter` 的谓词接收“迭代元素的引用”，所以收到 `&&u32`。
- `|&&n|` 是解构模式，剥掉两层引用，得到 `u32` 值。
- 因为 `u32` 实现了 `Copy`，这里可以复制数值进行计算。

这个模式并不能对所有非 `Copy` 类型照搬。

## 8. Arc 为什么能跨线程，Rc 为什么不行

`Rc<T>` 和 `Arc<T>` 都提供引用计数式共享所有权，区别在于计数的同步机制。

| 类型 | 所有权 | 典型用途 |
| --- | --- | --- |
| `Box<T>` | 独占所有权 | 堆分配、递归类型 |
| `Rc<T>` | 共享所有权，非原子计数 | 单线程共享 |
| `Arc<T>` | 共享所有权，原子计数 | 跨线程共享 |

多个线程同时克隆、销毁句柄，会并发读写引用计数。`Rc` 没有为这种情况提供同步，所以不能在线程间传递；`Arc` 使用原子操作管理计数。

原子操作保证这些计数更新不会因并发而丢失。它有一定成本，单线程程序并不是一律应该把 `Rc` 换成 `Arc`。

标准的 `Arc<T>` 要实现 `Send` 和 `Sync`，通常要求 `T: Send + Sync`：

- `Send`：值可以安全地转移到另一个线程。
- `Sync`：共享引用可以安全地在线程间使用。

`Vec<u32>` 满足这些条件，因此题目可以共享它。

## 9. Arc 不等于内部数据自动线程安全

`Arc` 解决的是“谁拥有数据、何时释放数据”，不是“怎样同步修改数据”。

多个线程只读同一个 `Vec<u32>`，像这道题一样，不需要额外的锁。但共享的 `Arc<Vec<u32>>` 不能直接让多个线程调用 `push`。

原因是 `push` 需要 `&mut Vec<_>`，而共享所有权不能凭空提供独占可变引用。

也不能用 `Arc<RefCell<T>>` 代替线程锁：`RefCell` 不实现 `Sync`，它的运行时借用检查不适用于跨线程共享。

常见组合：

| 需求 | 常见选择 |
| --- | --- |
| 多线程共享只读数据 | `Arc<T>` |
| 多线程互斥修改数据 | `Arc<Mutex<T>>` |
| 允许多个读者，写入时独占 | `Arc<RwLock<T>>` |
| 共享简单计数器 | `Arc<AtomicUsize>` 等原子类型 |

## 10. Arc<Mutex<T>> 示例

```rust
use std::sync::{Arc, Mutex};
use std::thread;

let counter = Arc::new(Mutex::new(0u32));
let mut handles = Vec::new();

for _ in 0..8 {
    let counter = Arc::clone(&counter);
    handles.push(thread::spawn(move || {
        let mut guard = counter.lock().unwrap();
        *guard += 1;
    }));
}

for handle in handles {
    handle.join().unwrap();
}

assert_eq!(*counter.lock().unwrap(), 8);
```

两层职责分开：

- `Arc`：允许多个线程拥有同一个 `Mutex`。
- `Mutex`：保证同一时刻只有一个持锁者访问被保护的数据。

`lock()` 返回的守卫 `MutexGuard` 在被销毁时自动解锁。上面在线程闭包结束时释放守卫，也就释放了锁。

这里用 `unwrap()` 简化示例。如果线程在持锁期间 panic，锁可能中毒，后续加锁会得到错误；实际项目需要按业务决定如何处理。

## 11. 不加锁时有没有修改的方式

有，但不等于允许多个线程同时修改同一份普通数据。

### 11.1 Arc::get_mut：确认没有其他句柄

当没有其他 `Arc` 或 `Weak` 指向同一分配时，`Arc::get_mut` 可以返回内部数据的独占可变引用，否则返回 `None`。

```rust
use std::sync::Arc;

let mut data = Arc::new(vec![1]);
Arc::get_mut(&mut data).unwrap().push(2);

let other = Arc::clone(&data);
assert!(Arc::get_mut(&mut data).is_none());
assert_eq!(&*other, &[1, 2]);
```

### 11.2 Arc::make_mut：写时复制

对支持克隆的内部数据，当存在其他强引用时，`Arc::make_mut` 会克隆内部数据，让当前句柄转向独立的数据后再提供可变引用。

```rust
use std::sync::Arc;

let mut a = Arc::new(vec![1]);
let b = Arc::clone(&a);

Arc::make_mut(&mut a).push(2);

assert_eq!(&*a, &[1, 2]);
assert_eq!(&*b, &[1]);
assert!(!Arc::ptr_eq(&a, &b));
```

因此，`make_mut` 不是“修改后所有共享者都能看到”，而是写时复制。

补充边界：如果没有其他强引用，只有弱引用，`make_mut` 会使这些弱引用与当前数据脱离关联，而不是因为弱引用存在就克隆内部数据。

## 12. Weak：观察数据，但不阻止它被销毁

`Weak<T>` 是弱引用，不增加强引用计数，不保证内部数据一直存在。

```rust
use std::sync::Arc;

let owner = Arc::new(String::from("hello"));
let observer = Arc::downgrade(&owner);

{
    let temporary_owner = observer.upgrade().unwrap();
    assert_eq!(temporary_owner.as_str(), "hello");
}

drop(owner);
assert!(observer.upgrade().is_none());
```

- `Arc::downgrade` 创建弱引用。
- `Weak::upgrade` 尝试获取一个新的 `Arc`。
- 如果内部数据已经被销毁，返回 `None`。

一个典型用途是避免强引用环。例如节点 A 强引用 B，B 又强引用 A，即使外部句柄全部释放，双方强引用计数也可能无法归零，造成内存泄漏。通常可把“反向关联”改成 `Weak`。

`Arc` 不是能够自动回收引用环的垃圾回收器。

## 13. 常见误区与总结

| 误区 | 正确理解 |
| --- | --- |
| `Arc::clone` 复制整个对象 | 只克隆共享所有权句柄 |
| `move` 会复制整个向量 | 这里移动的是一个 `Arc` 句柄 |
| `Arc` 让所有内部类型都线程安全 | 内部类型仍必须满足相应的 `Send`、`Sync` 条件 |
| 用 `Arc` 就能随意修改内部数据 | 共享修改需要锁、原子类型等机制 |
| 主线程释放句柄，子线程就不能使用数据 | 子线程自己的强引用仍然让数据保持存活 |
| `'static` 约束要求值永不销毁 | 约束借用关系，不要求实际存活到程序结束 |
| `Arc` 能解决所有内存泄漏 | 强引用环仍可能泄漏，需要考虑 `Weak` |

对应 `arc1.rs`，最重要的三句话：

1. `Arc::new(numbers)` 将向量交给共享所有权机制管理。
2. `Arc::clone(&shared_numbers)` 为每个线程创建句柄，不复制向量元素。
3. `move` 将该句柄交给子线程，最后一个强引用释放时才销毁共享数据。
