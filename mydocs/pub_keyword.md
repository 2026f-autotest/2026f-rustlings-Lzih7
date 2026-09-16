# Rust 中 `pub` 关键字讲解

## 1. 先给结论

`pub` 的作用是控制可见性，也就是“这个东西能不能被外部访问”。

在 Rust 中，默认规则非常重要：

- 模块中的函数、结构体、枚举、常量等，默认都是私有的
- 如果希望模块外部可以访问，就要显式写 `pub`

所以可以先记一句话：

`pub` 不是“定义一个东西”，而是“把这个东西对外公开”。

## 2. 为什么 Rust 需要 `pub`

Rust 的模块系统强调封装。

也就是说，一个模块内部可以有很多实现细节，但外部不一定应该看到全部内容。这样做有几个好处：

- 隐藏内部实现，减少误用
- 只暴露必要接口，代码边界更清晰
- 方便后续重构，不容易影响外部调用者

因此，Rust 采用的是“默认私有，按需公开”的设计。

## 3. 不写 `pub` 会怎样

先看一个最简单的例子：

```rust
mod a {
    fn hello() {
        println!("hello");
    }
}

fn main() {
    a::hello();
}
```

这段代码会报错，因为 `hello` 默认是私有函数，模块 `a` 外部不能调用它。

如果改成：

```rust
mod a {
    pub fn hello() {
        println!("hello");
    }
}

fn main() {
    a::hello();
}
```

就可以正常访问了。

## 4. 结合 `modules1.rs` 理解

你刚做的 `modules1.rs` 就是一个很标准的例子：

```rust
pub mod sausage_factory {
    fn get_secret_recipe() -> String {
        String::from("Ginger")
    }

    pub fn make_sausage() {
        get_secret_recipe();
        println!("sausage!");
    }
}

fn main() {
    sausage_factory::make_sausage();
}
```

这里有两层可见性：

- `pub mod sausage_factory`：模块本身对外可见
- `pub fn make_sausage()`：函数对模块外可见

而：

```rust
fn get_secret_recipe() -> String
```

没有写 `pub`，所以它仍然是模块内部私有函数，只能在 `sausage_factory` 内部使用。

这正符合题目的意思：外面可以“做香肠”，但看不到“秘密配方”。

## 5. `pub` 可以修饰哪些东西

`pub` 可以修饰很多项，常见的有：

- 函数：`pub fn`
- 模块：`pub mod`
- 结构体：`pub struct`
- 枚举：`pub enum`
- 常量：`pub const`
- 类型别名：`pub type`
- trait：`pub trait`

例如：

```rust
pub struct User {
    pub name: String,
    age: u32,
}
```

这里要特别注意：

- `User` 这个结构体本身是公开的
- 字段 `name` 是公开的
- 字段 `age` 没有写 `pub`，所以仍然是私有的

这说明“结构体公开”不等于“结构体所有字段都公开”。

## 6. 一个最容易错的点：结构体和字段的 `pub` 是分开的

看下面的例子：

```rust
mod people {
    pub struct User {
        pub name: String,
        age: u32,
    }
}

fn main() {
    let u = people::User {
        name: String::from("Alice"),
        age: 18,
    };
}
```

这段代码仍然会报错，因为虽然 `User` 是公开的，但 `age` 字段是私有的，模块外不能直接构造它。

这说明：

- `pub struct` 只表示类型名对外可见
- 字段要不要对外可见，还要单独看字段前面有没有 `pub`

## 7. 枚举为什么稍微特殊

对于枚举，情况和结构体不完全一样。

```rust
pub enum Message {
    Quit,
    Move,
}
```

如果枚举本身是 `pub`，那么它的各个变体通常也可以在外部使用。

所以枚举和结构体的一个区别是：

- `pub struct` 不会自动让所有字段公开
- `pub enum` 会让外部可以使用它的变体

这是做题时很容易混淆的点。

## 8. `pub` 的本质是“相对某个作用域公开”

很多初学者会把 `pub` 理解成“全世界都能访问”，这个理解不够准确。

更准确地说，`pub` 是让一个项在更外层的可见范围内可访问，但前提是访问路径上的每一层也都必须可见。

例如：

```rust
mod outer {
    mod inner {
        pub fn f() {}
    }
}

fn main() {
    outer::inner::f();
}
```

这里即使 `f` 是 `pub`，也不代表一定能从外部访问成功，因为 `inner` 本身不是公开模块。

所以要记住一条规则：

想从外部访问 `a::b::c`，路径上的每一层都必须是可访问的。

## 9. `pub(crate)`、`pub(super)`、`pub(in path)` 是什么

Rust 不只有最普通的 `pub`，还支持更细粒度的可见性控制。

### `pub(crate)`

表示“在当前 crate 内公开”。

```rust
pub(crate) fn helper() {}
```

含义是：

- 当前工程内部其他模块可以访问
- 工程外部不能访问

如果你把一个项目做成库，`pub(crate)` 很适合用来暴露“库内部共用接口”，但不想对库用户公开。

### `pub(super)`

表示“只对父模块公开”。

```rust
pub(super) fn helper() {}
```

含义是：

- 当前模块的父模块可以访问
- 更外部的其他地方不一定能访问

这适合父模块需要调用、但你又不想彻底对外开放的情况。

### `pub(in path)`

表示“只在某个指定路径内公开”。

```rust
pub(in crate::network) fn helper() {}
```

含义是：

- 只有 `crate::network` 这个范围内可以访问
- 范围之外不行

这是更精细的权限控制方式。

## 10. 一张对比表

| 写法 | 含义 |
| --- | --- |
| `fn f()` | 仅当前模块可见 |
| `pub fn f()` | 对外公开 |
| `pub(crate) fn f()` | 当前 crate 内公开 |
| `pub(super) fn f()` | 对父模块公开 |
| `pub(in path) fn f()` | 对指定路径范围公开 |

## 11. 做题时的判断方法

当你看到“某个函数调用报权限错误”时，可以按这个顺序检查：

1. 被调用的函数本身有没有写 `pub`
2. 它所在的模块有没有对外可见
3. 访问路径中每一层模块是否都可见
4. 如果是结构体字段，字段本身有没有写 `pub`
5. 是否其实应该用 `pub(crate)` 或 `pub(super)`，而不是直接全公开

## 12. 一个最实用的理解框架

可以把模块想成“房间”，把 `pub` 想成“开门权限”。

- 不写 `pub`：门关着，只有房间里的人能用
- 写 `pub`：外面的人也可以进来
- 写 `pub(crate)`：只对本栋楼里的人开放
- 写 `pub(super)`：只对上一层管理者开放

当然，这只是辅助理解。真正做题时，还是要回到“模块边界”和“访问路径”上来判断。

## 13. 最后记一句最重要的话

Rust 的可见性规则核心不是“怎么公开”，而是“默认私有，按需暴露”。

所以写代码时，通常不是先想“要不要都加 `pub`”，而是先想：

这个接口真的需要让外部访问吗？
