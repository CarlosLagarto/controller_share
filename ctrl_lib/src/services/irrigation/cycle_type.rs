use serde::{Deserialize, Serialize};
use num_enum::UnsafeFromPrimitive;

// o tema aqui com este tipo é para atingir 2 objetivos:
// - acabar com as comparações com strings que são menos eficientes
// - assegurar a ordenação dos ciclos qd se vão buscar á bd, para assegurar que os ciclos standards aparecem sempre depois dos internos, por causa das remoções e adições
//   que podem assim ser feitas com swap_remove, que é O(1) na remoção, em vez de O(n)... não que com vetores de 4 elementos isso faça muita diferença :-) mas é o principio
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, UnsafeFromPrimitive, Serialize, Deserialize)]
#[repr(u8)]
pub enum CycleType {
    Wizard = 0,
    Compensation = 1,
    Direct = 2,
    Standard = 3,
}

#[allow(clippy::derivable_impls)]
impl Default for CycleType {
    #[inline]
    #[rustfmt::skip]
    fn default() -> Self { CycleType::Standard }
}
