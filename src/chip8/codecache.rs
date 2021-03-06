extern crate memmap;

use std::mem;

use self::memmap::{Mmap, Protection};

use chip8::MEMORY_SIZE;
use chip8::ROM_START_ADDRESS;
use chip8::codeemitter::CodeEmitter;

const CACHE_CAPACITY: usize = 0x10000;

pub struct CodeCache {
	pub x86_block_addresses: [u32; MEMORY_SIZE],
	cache: Mmap,
	cache_size: usize
}

impl CodeCache {
	pub fn new(register_pc: &u16) -> CodeCache {
		let mut code_cache = CodeCache {
			x86_block_addresses: [0; MEMORY_SIZE],
			cache: Mmap::anonymous(CACHE_CAPACITY, Protection::ReadWrite).unwrap(),
			cache_size: 0
		};

		for address in ROM_START_ADDRESS..MEMORY_SIZE as u16 {
			let mut code_emitter = CodeEmitter::new();
			code_emitter.mov_imm_to_m16(address, register_pc);
			code_emitter.ret();
			code_cache.insert(address, code_emitter.raw_code);
		}

		code_cache
	}

	pub fn insert(&mut self, address: u16, block: Vec<u8>) {
		let new_size = self.cache_size + block.len();
		if new_size > CACHE_CAPACITY {
			panic!("Cache overflow");
		}
		let _ = self.cache.set_protection(Protection::ReadWrite);
		unsafe { 
			self.cache.as_mut_slice()[self.cache_size..new_size].copy_from_slice(&block);
			self.x86_block_addresses[address as usize] = self.cache.ptr().offset(self.cache_size as isize) as u32;
		}
		let _ = self.cache.set_protection(Protection::ReadExecute);
		self.cache_size = new_size;
	}

	pub fn execute(&self, address: u16) {
		let f: fn() = unsafe { mem::transmute(self.x86_block_addresses[address as usize]) };
		f();
	}
}
