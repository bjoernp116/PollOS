# FAT16 File system layout:

## Boot Record:

| Offset | Size | Usage |
|:------:|:----:|:------|
| 0 | 3 | Always: * 3C 90, jump over data! |
| 3 | 8 | OEM Identifier, first 8 Bytes is version of DOS |
| 11 | 2 | Number of bytes per sector (little endian) |
| 13 | 1 | Number of sectors per cluster |
| 14 | 2 | Number of reserved sectors (bootsector included) |
| 16 | 1 | Number of FATs, often 2 | 
| 17 | 2 | Number of root directory entries | 
| 19 | 2 | The total sectors in the logical volume (if 0: more than 65535 sectors see Large Sector Count at 0x20) |
| 21 | 1 | This byte indicates the Media Descriptor Type |
| 22 | 2 | Number of sectors per FAT | 
| 24 | 2 | Number of sectors per track |
| 26 | 2 | Number of heads or sides on the storage media | 
| 28 | 4 | Number of hidden sectors | 
| 32 | 4 | Large Sector Count | 
| 36 | 1 | Drive number |
| 37 | 1 | Reserved (Windows NT) |
| 38 | 1 | Signature (must be 0x28 or 0x29) | 
| 39 | 4 | VolumeID 'serial' number. Used for tracking volumes between computers. | 
| 43 | 11 | Volume Label String. Padded with spaces | 
| 54 | 8 | System ID string. String representation FAT file system type, padded with spaces, dont trust! | 
| 62 | 488 | Boot code | 
| 510 | 2 | Magic Number 0xAA55 | 

## Directory Entry

| Offset | Size | Usage |
|:------:|:----:|:------|
| 0 | 8 | 8.3 file name |
| 8 | 3 | 8.3 extension |
| 11 | 1 | Attributes |
| 12 | 1 | Reserved (Windows NT) | 
| 13 | 1 | Creation time in hundreds | 
| 14 | 2 | The time of creation H: 5bts, M: 6bts, S: 5bts*2 |
| 16 | 2 | The date of creation Y: 7bts, M: 4bts, D: 5bts |
| 18 | 2 | Last Accessed date (same format as creation date) |
| 20 | 2 | Always zero |
| 22 | 2 | Last modification time (same format as creation time) |
| 24 | 2 | Last modification date (same format as creation date) |
| 26 | 2 | This entry's first cluster number | 
| 28 | 4 | File size in bytes | 

**Long File Name**
If attributes == 0x0F the entry uses LFN (Long File Name). 

| Offset | Size | Usage |
|:------:|:----:|:------|
| 0 | 1 | Order of this entry in the sequence of file name entries |
| 1 | 10 | The first 5 charachters of the name (char = 2 bytes) |
| 11 | 1 | Attribute. Always 0x0F if LFN is used |
| 12 | 1 | Long entry type. Zero for name entries. |
| 13 | 1 | Checksum |
| 14 | 12 | The next 6 charachters | 
| 26 | 2 | Always zero |
| 28 | 4 | The final 2 charachters | 




## References

- [https://osdev.wiki/wiki/FAT#Long_File_Names](OSDev FAT)










