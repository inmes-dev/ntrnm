# 读访问器(Getter)的名称遵循 Rust 的命名规范(C-GETTER)

除了少数例外，在 Rust代码中 get 前缀不用于 Getter。
```rust
pub struct S {
    first: First,
    second: Second,
}

impl S {
    // 而不是 get_first
    pub fn first(&self) -> &First {
        &self.first
    }

    // 而不是 get_first_mut，get_mut_first，or mut_first
    pub fn first_mut(&mut self) -> &mut First {
        &mut self.first
    }
}
```

至于上文提到的少数例外，如下：当有且仅有一个值能被 Getter 所获取时，才使用 get 前缀。例如，Cell::get 能直接访问到 Cell 中的内容。

有些 Getter 会在过程中执行运行时检查，那么我们就可以考虑添加 _unchecked Getter 函数，这个函数虽然不安全，但是往往具有更高的性能。 典型的例子如下：

- fn get(&self, index: K) -> Option<&V>;
- fn get_mut(&mut self, index: K) -> Option<&mut V>;
- unsafe fn get_unchecked(&self, index: K) -> &V;
- unsafe fn get_unchecked_mut(&mut self, index: K) -> &mut V;

## 标准库示例

- std::io::Cursor::get_mut
- std::ptr::Unique::get_mut
- std::sync::PoisonError::get_mut
- std::sync::atomic::AtomicBool::get_mut
- std::collections::hash_map::OccupiedEntry::get_mut
- <[T]>::get_unchecked

# 命名要使用一致性的词序(C-WORD-ORDER)

这是一些标准库中的错误类型:

- JoinPathsError
- ParseBoolError
- ParseCharError
- ParseFloatError
- ParseIntError
- RecvTimeoutError
- StripPrefixError

它们都使用了 谓语-宾语-错误 的词序，如果我们想要表达一个网络地址无法分析的错误，由于词序一致性的原则，命名应该如下 ParseAddrError，而不是 AddrParseError。

词序和个人习惯有很大关系，想要注意的是，你可以选择合适的词序，但是要在包的范畴内保持一致性，就如标准库中的包一样。