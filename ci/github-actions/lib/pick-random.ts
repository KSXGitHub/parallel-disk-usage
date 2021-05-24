export function* pickRandom<Item>(source: readonly Item[], quantity: number) {
  let pool = [...source]
  while (quantity && pool.length) {
    const index = Math.floor(Math.random() * pool.length)
    yield pool[index]
    pool = [...pool.slice(0, index), ...pool.slice(index + 1)]
    quantity -= 1
  }
}

export default pickRandom
