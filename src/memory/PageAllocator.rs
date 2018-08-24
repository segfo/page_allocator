// ベアメタルで使うなら
// std::mem や std::ptr　を
// core::mem や core::ptr に書き換える
use std;

// 管理構造体の個数。実際に使うときは4000個くらい持っておけばよさそう。
const ARRAY_CNT:usize=0x10;

// メモリエリアを表現する構造体（双方向リスト）
// 以下、エンティティと呼ぶ
#[derive(Clone,Copy,Debug)]
struct MemoryArea{
    start:usize,
    size:usize,
    prev:MemoryAreaPtr,
    next:MemoryAreaPtr
}

// 管理構造体の静的な初期化
static mut MEMORY_AREAS:[MemoryArea;ARRAY_CNT]=[
        MemoryArea{
            start:0,size:0,
            prev:MemoryAreaPtr(std::ptr::null_mut()),
            next:MemoryAreaPtr(std::ptr::null_mut())
        };ARRAY_CNT];

#[derive(Clone,Copy,Debug,PartialEq)]
struct MemoryAreaPtr(*mut MemoryArea);

// ページサイズ。変更しちゃダメ。
const PAGE_SIZE:usize=0x1000;
pub struct PageMemory{
    mem:*mut u8,
    pages:usize
}
impl PageMemory{
    pub fn memory(&self)->*mut u8{
        self.mem
    }
    pub fn pages(&self)->usize{
        self.pages
    }
}

#[derive(Debug)]
pub struct PageMemoryManager{
    freelist:MemoryAreaPtr,
    uselist:MemoryAreaPtr,
    total_size:usize,
    lost_total:usize,
}

impl PageMemoryManager{
    unsafe fn trans_ptr(area:&mut MemoryArea)->MemoryAreaPtr{
        let ptr=std::mem::transmute::<&mut MemoryArea,*mut MemoryArea>(area);
        MemoryAreaPtr(ptr)
    }

    pub fn new()->Self{
        let mut manager=PageMemoryManager{
            freelist:MemoryAreaPtr(std::ptr::null_mut()),
            uselist:MemoryAreaPtr(std::ptr::null_mut()),
            total_size:0,
            lost_total:0
        };
        manager.list_init();
        let entity = unsafe{manager.get_entity()};
        manager.uselist=entity;
        manager
    }

    pub fn get_freearea_bytes(&self)->usize{
        self.total_size
    }

    fn list_init(&mut self){
        let max=unsafe{MEMORY_AREAS.len()-1};
        for i in 1..max{
            unsafe{
                MEMORY_AREAS[i].next=PageMemoryManager::trans_ptr(&mut MEMORY_AREAS[i+1]);
                MEMORY_AREAS[i].prev=PageMemoryManager::trans_ptr(&mut MEMORY_AREAS[i-1]);
            }
        }
        unsafe{
            MEMORY_AREAS[1].prev=PageMemoryManager::trans_ptr(&mut MEMORY_AREAS[0]);
            MEMORY_AREAS[0].next=PageMemoryManager::trans_ptr(&mut MEMORY_AREAS[1]);
            MEMORY_AREAS[max].prev=PageMemoryManager::trans_ptr(&mut MEMORY_AREAS[max-1]);
        }
        self.freelist=unsafe{PageMemoryManager::trans_ptr(&mut MEMORY_AREAS[0])};
    }

    unsafe fn get_entity(&mut self)->MemoryAreaPtr{
        let null=MemoryAreaPtr(std::ptr::null_mut());
        let entity=self.freelist;
        if (*entity.0).next!=null{
            let entity=self.freelist;
            self.freelist=(*entity.0).next;
            (*entity.0).prev=MemoryAreaPtr(std::ptr::null_mut());
            (*(*entity.0).next.0).prev=MemoryAreaPtr(std::ptr::null_mut());
            (*entity.0).next=MemoryAreaPtr(std::ptr::null_mut());
            entity
        }else{
            null
        }
    }
    // エンティティの開放（使い終わったものを管理テーブルに戻す）
    unsafe fn free_entity(&mut self,entity:MemoryAreaPtr){
        (*(*entity.0).prev.0).next = (*entity.0).next;
        if (*entity.0).next.0!=std::ptr::null_mut(){
            (*(*entity.0).next.0).prev = (*entity.0).prev;
        }

        (*entity.0).start=0;
        (*entity.0).size=0;
        // 0.|    (null)<-prev-{freelist.0}<-prev/next->{next}                  |
        // 1.|    {entity}<-prev-{freelist.0}<-prev/next->{next}                |
        (*self.freelist.0).prev=entity;
        // 2.|    {entity}<-prev/next->{freelist.0}-next->{next}                |
        (*entity.0).next=self.freelist;
        // 3.|    (null)<-prev-{entity}<-prev/next->{freelist.0}-next->{next}   |
        (*entity.0).prev=MemoryAreaPtr(std::ptr::null_mut());
        // 3.|    (null)<-prev-{freelist.0}<-prev/next->{next}-next->{next}     |
        self.freelist.0=entity.0;
    }
    // 
    pub unsafe fn free_frames(&mut self,page:PageMemory){
        let mut list = self.uselist;
        let free_start = page.mem as usize;
        let size = page.pages*PAGE_SIZE;
        let mut current = list;
        loop{
            if (*list.0).start > free_start{
                break;
            }
            current = list;
            list = (*list.0).next;
            if list.0 == std::ptr::null_mut(){break;}
        }
        let next=(*current.0).next;
        // とりあえず挿入
        let free_area = self.get_entity();
        // 管理構造体の空きがなくなってしまった。
        if free_area.0==std::ptr::null_mut(){
            self.lost_total+=size;
            return;
        }
        // メモリの空き合計サイズを増やす
        self.total_size+=page.pages()*PAGE_SIZE;
        // 空きエリア情報を初期化する
        (*free_area.0).start = free_start;
        (*free_area.0).size = size;
        (*current.0).next=free_area;
        (*free_area.0).prev=current;
        (*free_area.0).next=next;
        if next.0!=std::ptr::null_mut(){
            (*next.0).prev=free_area;
        }

        // 結合できる場合(前)
        let prev=*(*free_area.0).prev.0;
        if prev.start + prev.size == (*free_area.0).start {
            (*free_area.0).start = (*(*free_area.0).prev.0).start;
            (*free_area.0).size += (*(*free_area.0).prev.0).size;
            let prev=(*free_area.0).prev;
            self.free_entity(prev);
        }
        // 結合できる場合(後)
        let current=*free_area.0;
        let not_null = (*free_area.0).next.0!=std::ptr::null_mut();
        
        if  not_null && current.start+current.size==(*next.0).start{
            (*free_area.0).size += (*next.0).size;
            // nextを線形リストから外す
            self.free_entity(next);
        }
    }

    pub unsafe fn allocate_frames(&mut self,require_size:usize)->Option<PageMemory>{
        let mut list = self.uselist;
        let size = (require_size + 0xfff) & !0xfff;
        loop{
            if list.0==std::ptr::null_mut()||(*list.0).size >= size{
                break;
            }
            list = (*list.0).next;
        }
        if list.0==std::ptr::null_mut(){
            return None;
        }
        // メモリの空き合計サイズを増やす
        self.total_size -= size;
        let page = PageMemory{
                        mem:std::mem::transmute::<usize,*mut u8>((*list.0).start),
                        pages:size/PAGE_SIZE
                    };
        if (*list.0).size-size > 0{
            (*list.0).size-=size;
            (*list.0).start+=size;
        }else{
            // 管理している領域のサイズがゼロになったら管理情報は不要なので破棄
            self.free_entity(list);
        }
        Some(page)
    }
}

// メモリアドレスとサイズからページメモリを生成するユーティリティ関数
pub fn memtranse(mem:usize,size:usize)->PageMemory{
    PageMemory{
        mem:unsafe{std::mem::transmute::<usize,*mut u8>(mem)},
        pages:((size + 0xfff) & !0xfff) / PAGE_SIZE
    }
}

/////////////////////////////////////////////
// デバッグ用関数
fn show_area(area:MemoryArea)->String{
    format!("MemoryArea {{ start: 0x{:x}, size: 0x{:x}, prev: {:?}, next: {:?} }}",
            area.start,area.size,area.prev,area.next)
}

fn show_list(mlist:MemoryAreaPtr){
    let mut list = mlist;
    loop{
        unsafe{
            if list.0==std::ptr::null_mut(){
                break;
            }
            println!("{:?} : {}",list,show_area(*list.0));
            list=(*list.0).next;
        }
    }
}

pub fn show_available_memory(manager:&PageMemoryManager){
    let list = manager.freelist;
    println!("===statistics start===");
    println!("===free===");
    show_list(list);
    println!("===using===");
    let list = manager.uselist;
    show_list(list);
    println!("===statistics end===\n");
}
