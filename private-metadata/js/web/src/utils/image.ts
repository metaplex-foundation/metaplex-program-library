
/**
 * depth: 1 - monochrome
 *        4 - 4-bit grayscale
 *        8 - 8-bit grayscale
 *       16 - 16-bit colour
 *       32 - 32-bit colour
 **/
export function drawArray(arr: Buffer, depth: number) {
  var offset, height, data, image;

  function conv(size: number) {
    return String.fromCharCode(size&0xff, (size>>8)&0xff, (size>>16)&0xff, (size>>24)&0xff);
  }

  offset = depth <= 8 ? 54 + Math.pow(2, depth)*4 : 54;
  height = Math.ceil(Math.sqrt(arr.length * 8/depth));

  //BMP Header
  data  = 'BM';                          // ID field
  data += conv(offset + arr.length);     // BMP size
  data += conv(0);                       // unused
  data += conv(offset);                  // pixel data offset

  //DIB Header
  data += conv(40);                      // DIB header length
  data += conv(height);                  // image height
  data += conv(height);                  // image width
  data += String.fromCharCode(1, 0);     // colour panes
  data += String.fromCharCode(depth, 0); // bits per pixel
  data += conv(0);                       // compression method
  data += conv(arr.length);              // size of the raw data
  data += conv(2835);                    // horizontal print resolution
  data += conv(2835);                    // vertical print resolution
  data += conv(0);                       // colour palette, 0 == 2^n
  data += conv(0);                       // important colours

  //Grayscale tables for bit depths <= 8
  if (depth <= 8) {
    data += conv(0);

    for (var s = Math.floor(255/(Math.pow(2, depth)-1)), i = s; i < 256; i += s)  {
      data += conv(i + i*256 + i*65536);
    }
  }

  //Pixel data
  data += String.fromCharCode.apply(String, arr);

  return btoa(data);
}

