mod exif_reader;

fn main() {
	let test = [ 0u8, 1, 2, 3, 4, 5 ];
	let exif = exif_reader::from_memory( &test );

    println!("exif = {}", exif);
}
