/* eslint-disable @typescript-eslint/no-explicit-any */
export function chunks(arr: any[], size: number) {
  return arr.reduce((acc: any[], val, ind) => {
    const subIndex = ind % size;
    if (!Array.isArray(acc[subIndex])) {
      acc[subIndex] = [val];
    } else {
      acc[subIndex].push(val);
    }
    return acc;
  }, []);
}
