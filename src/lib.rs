#![no_std]
// #![feature(portable_simd)]
// #![warn(internal_features)]
// #![feature(core_intrinsics)]
use core::arch;
use core::arch::x86_64::__m128i;
use core::ptr::null_mut;
extern crate alloc;
use alloc::{string::ToString, vec::Vec};
use alloc::string::String;


#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Error(&'static str);

impl Error {
    pub const ERROR_TZM_EMPTY: Error = Error("特征码为空");
    pub const ERROR_TZM: Error = Error("错误的特征码");
    pub const ERROR_TZM_NOT_GET_INDEX: Error = Error("特征码下标未记录");
    pub const ERROR_TZM_NOT_FIND: Error = Error("特征码未找到");
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.0 {
            "特征码为空" => write!(f, "Error: 特征码为空"),
            "错误的特征码" => write!(f, "Error: 错误的特征码"),
            "特征码未找到" => write!(f, "Error: 特征码未找到"),
            _ => write!(f, "Error: Unknown"),
        }
    }
}

impl core::error::Error for Error {
    
}

/// This function is used to find a given pattern in a buffer.
///
/// # Parameters
///
/// * `tzm`: A string representing the pattern to be found. The pattern can contain '?' as wildcard characters.
/// * `buf_mask_u8`: A mutable reference to a vector of unsigned 8-bit integers to store the mask for each byte in the pattern.
/// * `buf_u8`: A mutable reference to a vector of unsigned 8-bit integers to store the bytes of the pattern.
/// * `index_vec`: A mutable reference to a vector of 64-bit integers to store the indices of the pattern bytes.
///
/// # Returns
///
/// * `Result`: A result indicating whether the pattern was found successfully or an error occurred.
///   - `Ok(())`: The pattern was found successfully.
///   - `Err(Error::ERROR_TZM_EMPTY)`: The pattern string was empty.
///   - `Err(Error::ERROR_TZM)`: An invalid character was found in the pattern string.
///   - `Err(Error::ERROR_TZM_NOT_GET_INDEX)`: The pattern indices could not be obtained.
pub fn find_tzm<'a>(tzm: &'a str, buf_mask_u8: &mut Vec<u8>, buf_u8: &mut Vec<u8>, index_vec: &mut Vec<i64>) -> Result<(), Error>{
    let mut buf = Vec::<char>::new();
    let text = tzm.to_string();
    if text.is_empty() { return Err(Error::ERROR_TZM_EMPTY); }
    let textch  = text.rsplit(' ').map(|s| s.chars());
    for chs in textch.rev() {
        for ch in chs.into_iter() {
            if ch != '?' && !((ch >= '0' && ch <= '9') || (ch >= 'A' && ch <= 'F') || (ch >= 'a' && ch <= 'f')) { return Err(Error::ERROR_TZM); }
            buf.push(ch);
        }

    }
    if buf.len() %2 !=0 { return Err(Error::ERROR_TZM)}
    let mut tzm_is_index : i64 = -1;
    for i in 0..buf.len() {
        let index = i * 2;
        if index > buf.len() - 2 {
            break;
        }
        let tmps = &buf[index..index+2];
        let tmp = tmps.iter().collect::<String>();
        let tmp = tmp.as_str();
        match tmp {
            "??" => {
                buf_mask_u8.push(0xff);
                buf_u8.push(0xff);
                tzm_is_index += 1;
                //index_vec.push(tzm_is_index);
            },
            _ => {
                if tmps[0] != '?' && tmps[1] == '?' {
                    buf_mask_u8.push(0xff >> 4);
                    let mut tmp_fx: [char; 2] = ['F'; 2];
                    tmp_fx[0] = tmps[0];
                    let tmp_fx = tmp_fx.iter().collect::<String>();
                    let tmp_fx = tmp_fx.as_str();
                    let tmp_fx =  u8::from_str_radix(tmp_fx, 16).unwrap();
                    buf_u8.push(tmp_fx);
                    tzm_is_index += 1;
                    //index_vec.push(tzm_is_index);
                } else if tmps[0] == '?' && tmps[1] != '?' {
                    buf_mask_u8.push(0xff << 4);
                    let mut tmp_xf: [char; 2] = ['F'; 2];
                    tmp_xf[1] = tmps[1];
                    let tmp_xf = tmp_xf.iter().collect::<String>();
                    let tmp_xf = tmp_xf.as_str();
                    let tmp_xf =  u8::from_str_radix(tmp_xf, 16).unwrap();
                    buf_u8.push(tmp_xf);
                    tzm_is_index += 1;
                    //index_vec.push(tzm_is_index);
                } else {
                    let tmp = u8::from_str_radix(tmp, 16).unwrap();
                    buf_u8.push(tmp);
                    buf_mask_u8.push(0x00);
                    tzm_is_index += 1;
                    index_vec.push(tzm_is_index);
                }    
            },
        }
    }
    if index_vec.len() == 0 { return Err(Error::ERROR_TZM_NOT_GET_INDEX)}
    Ok(())

}

/// This function is used to find a given pattern in a offset or a base + offset .
///
/// # Parameters
///
/// * `ret_list`: Out A (offset or a base + offset) list [`Vec<u64>`].
/// * `search_start_addr`: A [`u64`] is start addr.
/// * `search_size`: A [`i64`] is you want search the size.
/// * `tzm`:  A [`str`] representing the pattern to be found. The pattern can contain '?' as wildcard characters.
/// * `offset_size`: A [`u64`] you can input base or 0 if input 0 ret_list entry = offset if input base_addr ret_list entry = base(offset_size) + offset.
/// * `search_num`: A [`u64`] is search count
///
/// # Returns
///
/// * `Result`: A result indicating whether the pattern was found successfully or an error occurred.
///   - `Ok(())`: The pattern was found successfully.
///   - `Err(Error::ERROR_TZM_NOT_FIND)`: not get tzm err.
pub fn sse2_pattern_find(ret_list: &mut Vec<u64>, search_start_addr: u64, search_size: i64, tzm: &str, offset_size: u64, search_num: u64) -> Result<(), Error> {
    if search_size == 0  && search_start_addr == 0 { return Err(Error::ERROR_TZM_NOT_FIND)}
    let mut realaddr = search_start_addr;
    if search_size < 0 && search_start_addr > search_size.abs() as u64 //searchSize可上负下正（以字节为单位）
    {
            realaddr = search_start_addr - search_size.abs() as u64;
    }

    let mut buf_mask_u8 = Vec::new();
    let mut buf_u8 = Vec::new();
    let mut index_vec = Vec::new();
    find_tzm(tzm,  &mut buf_mask_u8, &mut buf_u8,  &mut index_vec)?;
    // buf_mask_u8.reverse();
    // buf_u8.reverse();
    // index_vec.reverse();
    // println!( "buf: {:x?}", buf);
    // println!( "buf_u8: {:x?}", buf_u8);
    // println!( "buf_mask_u8: {:x?}", buf_mask_u8);
    // println!( "index_v : {:x?}", index_vec);

    // 常规变量
    let max_address: *mut u8 = (realaddr + search_size.abs() as u64) as *mut u8;
    let  mut base_address= null_mut();
    let mut curr_address: *mut u8 = null_mut();
    let curr_pattern: *mut u8 = buf_u8.as_mut_ptr(); // 特征码字节序列的首地址
    let  mut curr_equal: u8;
    let mut curr_ptnch: u8;

    // 这是从未阅读的警告 解决方案
    let (_, _) = (base_address, curr_address);

    // SSE加速相关变量
    //__m128i ptnHead = _mm_set1_epi8(vecPtn.at(vecIdx.at(0)));  
    // let ptn_haad: __m128i = unsafe { arch::x86_64::_mm_loadu_si128(buf_u8.as_ptr() as _) };
    // println!( "ptn_head: {:x?}", ptn_haad );
    let ptn_haad: __m128i = unsafe { arch::x86_64::_mm_set1_epi8(buf_u8[index_vec[0] as usize] as _) };
    //println!( "ptn_head: {:x?}", ptn_haad );


    //__m128i headMsk = _mm_set1_epi8(vecMsk.at(vecIdx.at(0)));
    // let head_maks: __m128i = unsafe { arch::x86_64::_mm_loadu_si128(buf_mask_u8.as_ptr() as _) };
    // println!( "head_mask: {:x?}", head_maks );
    let head_maks: __m128i = unsafe { arch::x86_64::_mm_set1_epi8(buf_mask_u8[index_vec[0] as usize] as _)};
    //println!( "head_mask: {:x?}", head_maks );

    //__m128i curHead, curComp;
    let mut cur_head: __m128i;
    let mut cur_comp: __m128i;


    //ULONG bitComp, idxComp;
    let mut bitcomp: u32 = 0;
    let mut idxcomp: u32 = 0;

    // 这是从未阅读的警告 解决方案
    let _ = bitcomp;

    let mut j: u32;
    let mut i = index_vec[0];
    while  i <= search_size-16  {
        // SSE 加速
        
        base_address = (realaddr as u64  + i as u64) as *mut __m128i;
        //println!("-------------------------------------------------------");

        cur_head = unsafe { arch::x86_64::_mm_loadu_si128(base_address as _) };
        //println!("bitcomp is ch {:x?}", cur_head);

        cur_head = unsafe { arch::x86_64::_mm_or_si128(cur_head, head_maks)};
        //println!("bitcomp is ch1 {:x?}", cur_head);

        cur_comp = unsafe { arch::x86_64::_mm_cmpeq_epi8(ptn_haad, cur_head) };
        //println!("bitcomp is chc {:x?}", cur_comp);

        bitcomp = unsafe { arch::x86_64::_mm_movemask_epi8(cur_comp) } as _;
       // println!("-------------------------------------------------------");
        j = 0;
        //println!("bitcomp is {:x}", bitcomp);
        //println!("{:x?}", cur_head);
        // let mut state =  ;

        //println!("state: {} inxcomp: {}", state, idxcomp);
        while bit_scan_forward(&mut idxcomp, bitcomp) {
            //println!("base {:?}", curr_address as u64);
            // println!("j {:?}", j);
            curr_address = (base_address as u64 + j as u64+ idxcomp as u64 - index_vec[0] as u64) as _;
            for ch in 0..index_vec.len() {
                if curr_address as u64 >= max_address as u64 - buf_u8.len() as u64 {
                    break;
                }
                //let index = index_vec[ch] as u32;
                curr_ptnch = unsafe { *(curr_pattern.add(index_vec[ch] as usize)) };
                //println!("字节序列 for index: {:x?}", curr_ptnch);

                let mask = buf_mask_u8[index_vec[ch] as usize];
                //println!("buf_mask for index : {:x?}",  mask);
                let current_byte = unsafe { *(curr_address.add(index_vec[ch] as usize)) };
                //println!("current byte: {:x?}", current_byte);
                curr_equal = (current_byte | mask) ^ curr_ptnch;
                //println!("是否为0: {:x?}", curr_equal);
                if curr_equal != 0 {

                    break;
                } else {
                    if ch +1  == index_vec.len() {
                        ret_list.push((curr_address as u64 - search_start_addr + offset_size) as _);
                        //println!("{:?}", curr_address as u64);
                        if search_num != 0 && ret_list.len() >= search_num as usize { return Ok(()); }
                        break;
                    } else {
                        continue;
                    }
                    
                }
            }



            // if retList.len() < search_num as usize {
            //     if retList.len() == 0 { return Err(Error::ERROR_TZM_NOT_FIND)}
            // } 
            idxcomp +=1;
            bitcomp >>= idxcomp;
            j += idxcomp;
        }
        i +=16;
    }
    

    Ok(())
}


/// This function performs a bit scan forward operation to find the index of the first set bit in a given mask.
///
/// # Parameters
///
/// * `index`: A mutable reference to a u32 variable where the index of the first set bit will be stored.
/// * `mask`: A u32 value representing the mask from which the first set bit will be found.
///
/// # Return Value
///
/// * `bool`: A boolean value indicating whether a set bit was found in the mask.
///   - `true`: A set bit was found in the mask.
///   - `false`: No set bit was found in the mask.
///
/// # Example
///
/// ```rust
/// let mut index: u32 = 0;
/// let mask: u32 = 0b00001000;
/// let result = bit_scan_forward(&mut index, mask);
/// assert_eq!(result, true);
/// assert_eq!(index, 3); // The first set bit in the mask is at index 3.
/// ```
fn bit_scan_forward(index: &mut u32, mask: u32) -> bool {
    if mask == 0 {
        false
    } else {
        *index = mask.trailing_zeros();
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    #[test]
    fn find_tzm_works() {
        let result = "00 ?1 ?? ff b8";
        let mut buf_mask_u8 = Vec::<u8>::new();
        let mut buf_u8 = Vec::<u8>::new();
        let mut index_vec = Vec::<i64>::new();
        find_tzm(result,  &mut buf_mask_u8, &mut buf_u8,  &mut index_vec).unwrap();
        // assert_eq!(buf, ['0', '0', '?','1', '?', '?', 'f', 'f', 'b', '8']);
        // assert_eq!(len, 10);
        assert_eq!(buf_mask_u8, [0x00, 0xf0, 0xff, 00, 00]);
        assert_eq!(buf_u8, [0x00, 0xf1, 0xff, 0xff, 0xb8]);
        assert_eq!(index_vec, [0, 3, 4]);
    }

    #[test]
    fn sse2_pattern_find_works() {
        let mut ret_list = Vec::<u64>::new();
        let mut buffer = vec![0u8; 0x1400fffff];
        buffer[24000000-1] = 0x7b;
        buffer[24000000] = 0x48;
        buffer[24000001] = 0x8b;
        buffer[24000002] = 0x31;
        buffer[24000003] = 0x1c;
        buffer[24000004] = 0x15;
        buffer[24000005] = 0x00;
        buffer[24000006] = 0x48;
        buffer[24000007] = 0x8b;

        buffer[34000000-1] = 0x7b;
        buffer[34000000] = 0x48;
        buffer[34000001] = 0x8b;
        buffer[34000002] = 0x31;
        buffer[34000003] = 0x1c;
        buffer[34000004] = 0x15;
        buffer[34000005] = 0x00;
        buffer[34000006] = 0x48;
        buffer[34000007] = 0x8b;
        // buffer[20] = 0x48;
        // buffer[21] = 0x8b;
        // buffer[22] = 0x31;
        // buffer[23] = 0x1c;
        // buffer[24] = 0x15;
        // buffer[25] = 0x00;
        // buffer[26] = 0x48;
        // buffer[27] = 0x8b;
        let search_start_addr = buffer.as_mut_ptr() as u64;
        let search_size =  84000000;
        let tzm = "?b 48 8b 3? ?c ?? ?? 48 8b ?? ?? ??";
        let offset_size = 0;
        let search_num = 2;
        sse2_pattern_find(&mut ret_list, search_start_addr, search_size, tzm, offset_size, search_num).unwrap();
        assert_eq!(ret_list,  [24000000-1, 34000000-1]);
        

    }
}
