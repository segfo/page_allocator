mod memory;
use memory::PageAllocator::{show_available_memory,memtranse,PageMemoryManager};

fn main() {
    unsafe{
        let manager = PageMemoryManager::get_instance();
        manager.free_frames(memtranse(0x100000,768*0x1000));
//        show_available_memory(&manager);
        manager.free_frames(memtranse(0x4d5000,843*0x1000));
        manager.free_frames(memtranse(0x820000,2016*0x1000));
        manager.free_frames(memtranse(0x1000000,241664*0x1000));
        manager.free_frames(memtranse(0x3c000000,32*0x1000));
        manager.free_frames(memtranse(0x3c020000,9574*0x1000));
        manager.free_frames(memtranse(0x3e75d000,1408*0x1000));
        manager.free_frames(memtranse(0x3ecdd000,351*0x1000));
        manager.free_frames(memtranse(0x3ee4e000,3840*0x1000));
        manager.free_frames(memtranse(0x3fd4e000,384*0x1000));
        manager.free_frames(memtranse(0x3ff32000,158*0x1000));
 //       show_available_memory(&manager);
        println!("free : {} bytes",manager.get_freearea_bytes());

        let mem1=manager.allocate_frames(0x2000).unwrap();
        //show_available_memory(&manager);
        println!("free : {} bytes",manager.get_freearea_bytes());

        let mem2=manager.allocate_frames(0x1000).unwrap();
        //show_available_memory(&manager);
        println!("free : {} bytes",manager.get_freearea_bytes());
        let mem3=manager.allocate_frames(0x1000).unwrap();
        //show_available_memory(&manager);
        println!("free : {} bytes",manager.get_freearea_bytes());
        println!("{:?}  pages:{}",mem1.memory(),mem1.pages());
        println!("{:?}  pages:{}",mem2.memory(),mem2.pages());
        println!("{:?}  pages:{}",mem3.memory(),mem3.pages());
    }
}
