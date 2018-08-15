mod memory;
use memory::PageAllocator::{show_available_memory,memtranse,PageMemoryManager};

fn main() {
    let mut manager = PageMemoryManager::new();
//        show_available_memory(&manager);
    
    unsafe{
        manager.free_page_frames(memtranse(0x10000,0x1000));
        show_available_memory(&manager);
        manager.free_page_frames(memtranse(0x14000,0x2000));
        show_available_memory(&manager);
        manager.free_page_frames(memtranse(0x13000,0x1000));
        show_available_memory(&manager);
        ////

        let mem1=manager.get_page_frames(0x2000).unwrap();
        show_available_memory(&manager);

        let mem2=manager.get_page_frames(0x1000).unwrap();
        show_available_memory(&manager);
        let mem3=manager.get_page_frames(0x1000).unwrap();
        show_available_memory(&manager);

        println!("{:?}  pages:{}",mem1.memory(),mem1.pages());
        println!("{:?}  pages:{}",mem2.memory(),mem2.pages());
        println!("{:?}  pages:{}",mem3.memory(),mem3.pages());
    }
}
