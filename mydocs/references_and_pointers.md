# Rust 中的引用与指针机制

## 1. 先给结论

Rust 中常见的“指向数据的东西”并不完全相同：

| 类型 | 含义 | 编译器是否检查有效性 | 解引用是否需要 `unsafe` |
| --- | --- | --- | --- |
| `&T` | 共享引用 | 是 | 否 |
| `&mut T` | 独占可变引用 | 是 | 否 |
| `*const T` | 只读意图的裸指针 | 基本不检查 | 是 |
| `*mut T` | 可写意图的裸指针 | 基本不检查 | 是 |
| `Box<T>` | 拥有堆上数据的智能指针 | 是 | 否 |
| `usize` | 能表示地址大小的无符号整数 | 不是指针 | 不能直接解引用 |

最重要的区别是：

- 引用带有 Rust 的有效性、对齐、生命周期和别名规则保证。
- 裸指针可以表示底层地址，但编译器不会替你证明解引用安全。
- `usize` 只是整数。即使它保存了某个地址的数值，也不自动拥有引用或指针的安全保证。

## 2. 值、地址与引用

看一个普通变量：

```rust
let value: u32 = 10;
```

可以把它抽象为：

```text
变量 value
    |
    +---- 某块内存
          地址：例如 0x1000
          内容：10
          类型：u32
```

下面创建一个共享引用：

```rust,ignore
let reference: &u32 = &value;
```

`reference` 不拥有 `value`，它借用了 `value`：

```text
reference: &u32 ----> value: u32 = 10
```

引用通常在机器层面包含地址信息，但 Rust 语言层面的引用不仅是一个数字地址。它还附带编译器能够依赖的保证，例如：

- 指向的数据有效
- 地址满足目标类型的对齐要求
- 在引用有效期间，数据不会被过早销毁
- 访问符合共享引用或可变引用的别名规则

因此，不能简单地把 Rust 引用理解成“包装过的整数地址”。

## 3. 共享引用 `&T`

共享引用使用 `&T` 表示：

```rust
let value = 10;
let a = &value;
let b = &value;

assert_eq!(*a, 10);
assert_eq!(*b, 10);
```

同一时间可以存在多个共享引用：

```text
a: &i32 ---+
           +----> value
b: &i32 ---+
```

共享引用的核心规则可以概括为：

“可以有多个读取者，但不能在这些共享借用有效时通过普通方式修改目标。”

这里的“共享”不是说所有类型都能跨线程共享。跨线程还要看类型是否实现 `Sync`。

### 3.1 `&` 与 `*`

在引用语境中：

- `&value` 创建引用，也叫借用
- `*reference` 解引用，访问引用指向的值

例如：

```rust
let value = 42;
let reference = &value;

assert_eq!(*reference, 42);
```

解引用安全引用不需要 `unsafe`，因为编译器已经检查了引用的有效性。

## 4. 可变引用 `&mut T`

可变引用允许修改借用的数据：

```rust
let mut value = 10;

{
    let reference: &mut i32 = &mut value;
    *reference += 5;
}

assert_eq!(value, 15);
```

创建可变引用需要满足两个条件：

1. 原变量允许修改，例如使用 `let mut`
2. 当前访问必须满足独占要求

最常用的规则是：

“同一时间，要么存在任意数量的共享引用，要么存在一个可变引用。”

例如，下面的代码不能通过编译：

```compile_fail
let mut value = 10;
let mutable = &mut value;
let shared = &value;

*mutable += 1;
println!("{shared}");
```

因为 `mutable` 后面还会使用，所以它的借用仍然有效；此时又创建了指向同一数据的共享引用。

### 4.1 独占不等于变量永远只有一个名字

`&mut T` 的“独占”是针对某段有效使用期而言的。一个可变引用最后一次使用后，借用可能提前结束：

```rust
let mut value = 10;

let reference = &mut value;
*reference += 1;

// reference 此后不再使用，它的借用可以在这里结束。
println!("{value}");
```

这称为非词法生命周期（Non-Lexical Lifetimes，NLL）。借用不一定持续到整个花括号作用域末尾。

## 5. 引用与生命周期

引用不能比它指向的数据活得更久：

```compile_fail
let reference;

{
    let value = String::from("hello");
    reference = &value;
}

println!("{reference}");
```

内部作用域结束后，`value` 被销毁。如果允许继续使用 `reference`，它就会成为悬垂引用。

Rust 使用生命周期分析在编译期阻止这种情况。生命周期标注描述引用之间的关系，不会延长数据真实的存活时间。

详细内容参见同目录的 `lifetimes.md`。

## 6. 裸指针 `*const T` 与 `*mut T`

Rust 有两种裸指针：

```rust
let mut value = 10u32;

let const_ptr: *const u32 = &value;
let mut_ptr: *mut u32 = &mut value;
```

- `*const T` 表示指向 `T` 的只读意图裸指针
- `*mut T` 表示指向 `T` 的可写意图裸指针

这里的 `const` 和 `mut` 描述通过该指针进行访问的能力，不等同于 C++ 中所有关于对象常量性的规则。

裸指针与安全引用相比有几个重要差异：

- 可以为空
- 可以悬垂
- 可以未对齐
- 可以指向未初始化或已经释放的内存
- 可以存在多个指向同一位置的可变裸指针
- 创建裸指针通常不需要 `unsafe`
- 解引用裸指针需要 `unsafe`

例如，创建空指针本身是安全的：

```rust
let pointer: *const u32 = std::ptr::null();
assert!(pointer.is_null());
```

但是解引用它会造成未定义行为，因此不能这样做：

```rust,ignore
unsafe {
    println!("{}", *pointer);
}
```

`unsafe` 不会把无效操作变安全，它只是表示程序员承担了证明安全条件的责任。

## 7. 为什么创建裸指针可以安全，解引用却不行

单纯保存一个地址不会立刻读取或写入该地址，因此通常不会立即破坏内存安全：

```rust
let pointer = std::ptr::null::<u32>();
```

真正危险的是通过指针访问内存：

```rust,ignore
unsafe {
    let value = *pointer;
}
```

解引用裸指针时，编译器无法自动证明以下条件：

- 指针非空
- 指针指向仍然存活的对象
- 地址对 `T` 正确对齐
- 内存中确实保存着有效的 `T`
- 读取或写入没有违反别名规则
- 写入位置可写
- 访问没有越界

因此，解引用裸指针必须进入 `unsafe` 块。

## 8. unsafe 的真正含义

`unsafe` 的意思不是：

“关闭 Rust 的所有检查。”

它真正表示：

“这小段代码执行了编译器无法证明安全的操作，由程序员负责保证其安全前提。”

即使在 `unsafe` 块内，以下规则仍然存在：

- 类型检查
- 所有权检查
- 大多数借用检查
- 生命周期检查

`unsafe` 只允许执行少数额外操作，例如：

- 解引用裸指针
- 调用 `unsafe fn`
- 访问或修改可变静态变量
- 实现 `unsafe trait`
- 访问 `union` 字段

因此，`unsafe` 是一份安全契约，不是跳过所有规则的开关。

## 9. 逐层拆解 tests5.rs 的调用

题目中的调用是：

```rust,ignore
unsafe { modify_by_address(&mut t as *mut u32 as usize) };
```

不要把它当成一个整体。它包含三步转换和一次函数调用。

假设：

```rust
let mut t: u32 = 0x12345678;
```

### 9.1 第一步：`&mut t`

```rust,ignore
&mut t
```

类型是：

```text
&mut u32
```

它创建了一个指向 `t` 的独占可变引用。此时 Rust 保证：

- `t` 有效
- 地址满足 `u32` 对齐
- 可以通过该可变引用修改 `t`
- 这段独占借用期间不能通过冲突方式访问 `t`

### 9.2 第二步：`as *mut u32`

```rust,ignore
&mut t as *mut u32
```

把安全的可变引用转换为可变裸指针：

```text
&mut u32  ->  *mut u32
```

创建裸指针本身不需要 `unsafe`。裸指针仍指向 `t`，但编译器不再像对 `&mut u32` 那样自动保证每次使用都安全。

### 9.3 第三步：`as usize`

```rust,ignore
&mut t as *mut u32 as usize
```

把裸指针转换为整数地址：

```text
&mut u32  ->  *mut u32  ->  usize
```

`usize` 的位宽足以表示当前平台上对象地址的数值：

- 32 位平台通常为 32 位
- 64 位平台通常为 64 位

但是 `usize` 只是整数，不能直接写：

```text
*address
```

因为编译器不知道这个整数应该指向什么类型，也不知道它是否真的是有效地址。

### 9.4 第四步：调用 unsafe 函数

```rust,ignore
unsafe { modify_by_address(address) };
```

`modify_by_address` 被声明为：

```rust,ignore
unsafe fn modify_by_address(address: usize)
```

调用者必须保证文档中的安全契约成立。这个题目中至少包括：

- `address` 来自一个仍然存活的 `u32`
- 地址满足 `u32` 的对齐要求
- 这块内存可写
- 写入期间拥有独占访问权

因为编译器无法仅根据 `usize` 验证这些事实，所以调用必须放进 `unsafe` 块。

## 10. 逐层拆解函数中的写入

函数体是：

```rust
unsafe fn modify_by_address(address: usize) {
    // SAFETY: 调用者保证 address 是有效、对齐且可独占写入的 u32 地址。
    unsafe {
        *(address as *mut u32) = 0xAABBCCDD;
    }
}
```

先转换：

```rust,ignore
address as *mut u32
```

类型变化为：

```text
usize -> *mut u32
```

然后：

```rust,ignore
*(address as *mut u32)
```

解引用这个裸指针，得到它指向的 `u32` 内存位置。

最后：

```rust,ignore
*(address as *mut u32) = 0xAABBCCDD;
```

向该位置写入新值。因此原变量 `t` 从：

```text
0x12345678
```

变成：

```text
0xAABBCCDD
```

完整数据流是：

```text
t: u32
   |
   | &mut t
   v
&mut u32
   |
   | as *mut u32
   v
*mut u32
   |
   | as usize
   v
usize 地址值
   |
   | as *mut u32
   v
*mut u32
   |
   | unsafe 解引用并写入
   v
t = 0xAABBCCDD
```

## 11. 为什么 unsafe fn 内部还要写 unsafe 块

题目中既有：

```rust,ignore
unsafe fn modify_by_address(...)
```

又有：

```rust,ignore
unsafe {
    *(address as *mut u32) = ...;
}
```

它们表达不同责任：

- `unsafe fn`：调用该函数需要满足额外安全契约
- `unsafe { ... }`：这里实际执行了需要人工证明安全的操作

在现代 Rust 的推荐风格中，`unsafe fn` 的函数体不会被视为“所有操作都可以无条件 unsafe”。配合 `unsafe_op_in_unsafe_fn` lint，函数内部的危险操作仍应放入明确的 `unsafe` 块。

这样做能缩小审查范围：读代码的人可以直接看到究竟哪一行依赖安全契约。

## 12. 安全注释应该写什么

下面这种注释信息不足：

```rust
// SAFETY: This is safe.
```

它没有解释为什么安全。

更合格的安全注释需要对应具体操作，说明哪些前提保证成立：

```rust,ignore
// SAFETY: `address` was produced from a live, uniquely borrowed `u32`.
// It is non-null, correctly aligned, writable, and valid for one `u32`.
unsafe {
    *(address as *mut u32) = 0xAABBCCDD;
}
```

对于 `unsafe fn`，还应通过 `# Safety` 文档明确告诉调用者必须保证什么：

```rust
/// Writes a new value to `address`.
///
/// # Safety
///
/// `address` must point to a live, aligned, writable `u32`. The caller must
/// have exclusive access to that value for the duration of the write.
unsafe fn modify_by_address(address: usize) {
    // ...
}
```

原则是：

- `# Safety` 写给调用者：调用前必须保证什么
- `// SAFETY:` 写给实现审查者：当前危险操作为什么满足要求

## 13. 对齐是什么

不同类型通常要求地址按一定字节边界排列。例如某个平台上的 `u32` 可能要求 4 字节对齐：

```text
可能对齐：0x1000、0x1004、0x1008
可能未对齐：0x1001、0x1002、0x1003
```

即使某个地址指向至少 4 字节可读内存，把未对齐地址直接解引用为 `*const u32` 仍可能造成未定义行为。

处理确实允许未对齐的数据时，应使用专门操作：

```rust
let bytes = [1u8, 0, 0, 0];
let pointer = bytes.as_ptr().cast::<u32>();

// SAFETY: `bytes` 至少包含 4 字节；read_unaligned 不要求 u32 对齐。
let value = unsafe { pointer.read_unaligned() };
assert_eq!(value, 1);
```

这里的数值结果还依赖机器字节序。示例中的断言适用于小端平台，因此跨平台协议代码通常应显式使用 `u32::from_le_bytes` 或 `u32::from_be_bytes`。

## 14. 裸指针与别名规则

裸指针可以被复制：

```rust
let mut value = 10;
let first: *mut i32 = &mut value;
let second = first;
```

但“能创建两个裸指针”不代表“任何交错读写方式都合法”。当裸指针来自引用时，仍要遵守 Rust 内存模型中关于来源、有效性和别名的规则。

一个实用的安全原则是：

- 进行共享读取时，不要同时修改同一数据
- 进行写入时，确保不存在会与它冲突的活动访问
- 不要在引用失效后继续使用由它派生出的指针
- 不要仅因为裸指针是 `Copy` 就认为任意复制和使用都安全

底层别名规则比一句“多个读或一个写”更精细；编写 `unsafe` 抽象时应优先使用标准库提供的安全 API，并使用 Miri 等工具辅助检查，但工具也不能代替完整的安全证明。

## 15. 指针算术与边界

裸指针支持地址偏移，但必须遵守分配边界：

```rust
let values = [10u32, 20, 30];
let pointer = values.as_ptr();

// SAFETY: pointer 指向包含 3 个 u32 的同一数组，add(1) 仍在边界内。
let second = unsafe { *pointer.add(1) };
assert_eq!(second, 20);
```

需要注意：

- `add(n)` 的单位是 `T`，不是字节
- `*const u32` 的 `add(1)` 通常前进一个 `u32` 的大小
- 计算和解引用都必须符合对应 API 的边界要求
- “尾后指针”有时可用于比较或计算，但不能解引用

不要用随意的整数加减代替指针 API。

## 16. 指针与整数互转的注意事项

题目为了教学演示了：

```rust,ignore
pointer as usize
```

以及：

```rust,ignore
address as *mut u32
```

这在简单、立即往返并满足平台规则的示例里可能工作，但真实底层代码还要考虑指针来源（provenance）。

现代 Rust 提供了用于显式处理地址的 API，例如：

```rust
let mut value = 10u32;
let pointer = &mut value as *mut u32;
let address = pointer.addr();
let restored = pointer.with_addr(address);

// SAFETY: restored 保留 pointer 的来源，并仍指向存活且独占的 value。
unsafe {
    *restored = 20;
}

assert_eq!(value, 20);
```

`addr()` 获取地址部分，`with_addr()` 使用已有指针的来源构造新地址。它比“丢弃指针、只保存一个整数，再凭空恢复”更明确。

此外，不应假设：

- 任意整数都能转成可解引用指针
- 指针转整数再转回在所有环境中都无条件保留全部语义
- 地址在程序运行期间永远不变
- 不同进程中的相同地址数值指向相同对象

FFI、内存映射、分配器、嵌入式寄存器和序列化地址等场景都有各自额外契约。

## 17. 空指针与 Option

裸指针可以为空：

```rust
let pointer: *const u32 = std::ptr::null();
```

安全引用 `&T` 和 `&mut T` 不能为 null。Rust 因此可以对某些类型进行空指针优化，例如：

```rust
use std::mem::size_of;

assert_eq!(size_of::<Option<&u32>>(), size_of::<&u32>());
```

`Option<&T>` 可以用引用类型中不允许出现的 null 表示 `None`，通常不需要额外空间。

但不要把这一实现优化推广为任意类型都必然具有同样布局。依赖具体 ABI 或布局时，应查阅对应类型的布局保证。

## 18. 智能指针与裸指针的区别

`Box<T>`、`Rc<T>`、`Arc<T>` 被称为智能指针，但它们不仅保存地址，还管理所有权：

| 类型 | 主要职责 |
| --- | --- |
| `Box<T>` | 独占拥有堆上的 `T` |
| `Rc<T>` | 单线程共享所有权 |
| `Arc<T>` | 多线程共享所有权 |
| `*const T` / `*mut T` | 底层地址访问，不自动管理所有权 |

例如：

```rust
let boxed = Box::new(42);
assert_eq!(*boxed, 42);
```

`Box<T>` 被销毁时会自动销毁 `T` 并释放堆内存。裸指针本身没有这种责任：

- 裸指针离开作用域不会自动释放它指向的内存
- 多个裸指针可能指向同一对象
- 裸指针不知道自己是否拥有目标

不要通过 `Box::from_raw` 接管一个并非由兼容分配方式产生、或者仍由其他所有者管理的地址，否则可能发生重复释放或分配器不匹配。

## 19. 常见未定义行为

以下操作即使有时“看起来能运行”，也可能是未定义行为：

### 19.1 解引用空指针

```rust,ignore
let pointer = std::ptr::null::<u32>();
unsafe { println!("{}", *pointer) };
```

### 19.2 使用悬垂指针

```rust,ignore
let pointer = {
    let value = Box::new(10u32);
    Box::into_raw(value)
};

// 只有在正确恢复并管理所有权时才能使用；错误释放后继续访问会悬垂。
```

### 19.3 解引用未对齐指针

把任意字节地址直接当成 `*const u32` 解引用。

### 19.4 越界访问

从数组指针偏移到分配范围外再进行读取或写入。

### 19.5 违反别名规则

在存在冲突访问时，通过裸指针修改同一对象。

### 19.6 读取无效位模式

并不是任意字节组合对任意 Rust 类型都有效。例如 `bool` 只允许有效布尔位模式；凭空把其他字节解释为 `bool` 可能造成未定义行为。

## 20. 为什么实际函数应优先接收 &mut u32

这个练习故意使用 `usize` 地址来训练 `unsafe`。但如果没有 FFI、硬件寄存器或底层内存接口等特殊要求，更合理的 API 是安全引用：

```rust
fn modify(value: &mut u32) {
    *value = 0xAABBCCDD;
}

let mut value = 0x12345678;
modify(&mut value);
assert_eq!(value, 0xAABBCCDD);
```

相比 `unsafe fn modify_by_address(address: usize)`，这个版本让编译器自动保证：

- 引用非空
- 地址正确对齐
- `u32` 仍然存活
- 调用期间拥有可变访问权

安全抽象的基本原则是：

“尽量让类型系统表达安全条件；只有类型系统确实无法表达时，才把最小范围的实现放进 `unsafe`。”

## 21. 考试式总结

Rust 的引用 `&T` 和 `&mut T` 是受编译器检查的借用。共享引用允许多个读取者，可变引用要求独占访问；两者都必须始终指向有效、对齐且仍然存活的数据。裸指针 `*const T` 和 `*mut T` 可以表示底层地址，但不携带同等的自动安全保证，创建裸指针通常是安全的，解引用裸指针必须使用 `unsafe`。`unsafe` 不会自动保证操作正确，而是把证明指针有效性、对齐、边界和别名规则的责任交给程序员。

对于题目中的：

```text
&mut t as *mut u32 as usize
```

类型变化依次是：

```text
u32
  -> &mut u32
  -> *mut u32
  -> usize
```

函数内部再执行：

```text
usize
  -> *mut u32
  -> unsafe 解引用
  -> 写入原变量 t
```

最应该记住的三句话：

1. `&T` 和 `&mut T` 是带安全保证的借用，裸指针只是底层访问工具。
2. `unsafe` 表示程序员必须证明安全条件成立，不表示危险代码自动变安全。
3. 能使用 `&mut T` 表达的接口，应优先使用安全引用，而不是传递整数地址。
