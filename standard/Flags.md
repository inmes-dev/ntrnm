# Flags/Enum 开发标准

为统一标识符的定义，例如在群聊群成员权限设定中，我们应该按照以下标准进行定义`flags`：

```rust
use bitflags::bitflags;

// The `bitflags!` macro generates `struct`s that manage a set of flags.
bitflags! {
    /// Represents a set of flags.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct GroupMemberPermission: u32 {
        /// The value `UPLOAD_FILE`, at bit position `0`.
        const UPLOAD_FILE =       0b00000001;
        /// The value `UPLOAD_PICTURE`, at bit position `1`.
        const UPLOAD_PICTURE =    0b00000010;
        /// The value `LAUNCH_TEMP_CHAT`, at bit position `2`.
        const LAUNCH_TEMP_CHAT =  0b00000100;
        const LAUNCH_GROUP_CHAT = 0b00001000;
        
        const VISIBLE_HISTORY_MSG_FOR_NEW_MEMBER = 0b00010000;
        
        const ALL_PERMISSION = Self::UPLOAD_FILE.bits() | Self::UPLOAD_PICTURE.bits() | Self::LAUNCH_TEMP_CHAT.bits() | Self::LAUNCH_GROUP_CHAT.bits() | Self::VISIBLE_HISTORY_MSG_FOR_NEW_MEMBER.bits();
    }
}

fn main() {
    let e1 = Flags::UPLOAD_FILE | Flags::LAUNCH_TEMP_CHAT;
    let e2 = Flags::UPLOAD_PICTURE | Flags::LAUNCH_TEMP_CHAT;
    assert_eq!((e1 | e2), Flags::ALL_PERMISSION);   // union
    assert_eq!((e1 & e2), Flags::LAUNCH_TEMP_CHAT);     // intersection
    assert_eq!((e1 - e2), Flags::UPLOAD_FILE);     // set difference
    assert_eq!(!e2, Flags::UPLOAD_FILE);           // set complement
}
```

## 优势

- 代码简洁，易于理解
- 代码可读性高
- 代码可维护性高
- 代码可扩展性高