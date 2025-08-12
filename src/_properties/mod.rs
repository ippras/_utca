//! [Fatty acid methyl ester viscosity](https://www.sciencedirect.com/science/article/pii/S0016236123000790)

// Critical Properties
pub mod critical;
pub mod density;
pub mod viscosity;

// // 2.7 * 48 + -31.95 + -13.28 * 0.07787616392032737
// // 6.79 * 48 + -19.09 + -36.7 * 0.07787616392032737
// // 44,5 = 317,65

// /// - [lipidlibrary.shinyapps.io](https://lipidlibrary.shinyapps.io/Triglyceride_Property_Calculator/)
// /// - [crcfoodandhealth.com](https://www.crcfoodandhealth.com/downloads/asmTPC-webMathe.pdf)
// pub(super) fn thermodynamic(&self, tag: Tag<usize>) -> Thermodynamic {
//     let formula = tag.map(|index| &self.state.entry().meta.formulas[index]);
//     Thermodynamic::new(formula)
// }
