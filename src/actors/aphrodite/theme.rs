// src/actors/aphrodite/theme.rs
// OLYMPUS v13 - Aphrodite Theme System

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScale {
    pub n50: String,
    pub n100: String,
    pub n200: String,
    pub n300: String,
    pub n400: String,
    pub n500: String,
    pub n600: String,
    pub n700: String,
    pub n800: String,
    pub n900: String,
    pub n950: String,
}

impl Default for ColorScale {
    fn default() -> Self {
        Self {
            n50: "#eff6ff".to_string(),
            n100: "#dbeafe".to_string(),
            n200: "#bfdbfe".to_string(),
            n300: "#93c5fd".to_string(),
            n400: "#60a5fa".to_string(),
            n500: "#3b82f6".to_string(),
            n600: "#2563eb".to_string(),
            n700: "#1d4ed8".to_string(),
            n800: "#1e40af".to_string(),
            n900: "#1e3a8a".to_string(),
            n950: "#172554".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontSizes {
    pub xs: String,
    pub sm: String,
    pub base: String,
    pub lg: String,
    pub xl: String,
    pub xl2: String,
    pub xl3: String,
    pub xl4: String,
    pub xl5: String,
    pub xl6: String,
    pub xl7: String,
    pub xl8: String,
    pub xl9: String,
}

impl Default for FontSizes {
    fn default() -> Self {
        Self {
            xs: "0.75rem".to_string(),
            sm: "0.875rem".to_string(),
            base: "1rem".to_string(),
            lg: "1.125rem".to_string(),
            xl: "1.25rem".to_string(),
            xl2: "1.5rem".to_string(),
            xl3: "1.875rem".to_string(),
            xl4: "2.25rem".to_string(),
            xl5: "3rem".to_string(),
            xl6: "3.75rem".to_string(),
            xl7: "4.5rem".to_string(),
            xl8: "6rem".to_string(),
            xl9: "8rem".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpacingScale {
    pub n0: String,
    pub n0_5: String,
    pub n1: String,
    pub n1_5: String,
    pub n2: String,
    pub n2_5: String,
    pub n3: String,
    pub n3_5: String,
    pub n4: String,
    pub n5: String,
    pub n6: String,
    pub n7: String,
    pub n8: String,
    pub n9: String,
    pub n10: String,
    pub n11: String,
    pub n12: String,
    pub n14: String,
    pub n16: String,
    pub n20: String,
    pub n24: String,
    pub n28: String,
    pub n32: String,
    pub n36: String,
    pub n40: String,
    pub n44: String,
    pub n48: String,
    pub n52: String,
    pub n56: String,
    pub n60: String,
    pub n64: String,
    pub n72: String,
    pub n80: String,
    pub n96: String,
}

impl Default for SpacingScale {
    fn default() -> Self {
        Self {
            n0: "0px".to_string(),
            n0_5: "0.125rem".to_string(),
            n1: "0.25rem".to_string(),
            n1_5: "0.375rem".to_string(),
            n2: "0.5rem".to_string(),
            n2_5: "0.625rem".to_string(),
            n3: "0.75rem".to_string(),
            n3_5: "0.875rem".to_string(),
            n4: "1rem".to_string(),
            n5: "1.25rem".to_string(),
            n6: "1.5rem".to_string(),
            n7: "1.75rem".to_string(),
            n8: "2rem".to_string(),
            n9: "2.25rem".to_string(),
            n10: "2.5rem".to_string(),
            n11: "2.75rem".to_string(),
            n12: "3rem".to_string(),
            n14: "3.5rem".to_string(),
            n16: "4rem".to_string(),
            n20: "5rem".to_string(),
            n24: "6rem".to_string(),
            n28: "7rem".to_string(),
            n32: "8rem".to_string(),
            n36: "9rem".to_string(),
            n40: "10rem".to_string(),
            n44: "11rem".to_string(),
            n48: "12rem".to_string(),
            n52: "13rem".to_string(),
            n56: "14rem".to_string(),
            n60: "15rem".to_string(),
            n64: "16rem".to_string(),
            n72: "18rem".to_string(),
            n80: "20rem".to_string(),
            n96: "24rem".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerSpacing {
    pub xs: String,
    pub sm: String,
    pub md: String,
    pub lg: String,
    pub xl: String,
    pub xl2: String,
}

impl Default for ContainerSpacing {
    fn default() -> Self {
        Self {
            xs: "1rem".to_string(),
            sm: "1.5rem".to_string(),
            md: "2rem".to_string(),
            lg: "3rem".to_string(),
            xl: "4rem".to_string(),
            xl2: "6rem".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionSpacing {
    pub sm: String,
    pub md: String,
    pub lg: String,
    pub xl: String,
    pub xl2: String,
}

impl Default for SectionSpacing {
    fn default() -> Self {
        Self {
            sm: "3rem".to_string(),
            md: "5rem".to_string(),
            lg: "8rem".to_string(),
            xl: "12rem".to_string(),
            xl2: "16rem".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowSystem {
    pub sm: String,
    pub normal: String,
    pub md: String,
    pub lg: String,
    pub xl: String,
    pub xl2: String,
    pub inner: String,
    pub none: String,
}

impl Default for ShadowSystem {
    fn default() -> Self {
        Self {
            sm: "0 1px 2px 0 rgb(0 0 0 / 0.05)".to_string(),
            normal: "0 1px 3px 0 rgb(0 0 0 / 0.1), 0 1px 2px -1px rgb(0 0 0 / 0.1)".to_string(),
            md: "0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)".to_string(),
            lg: "0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1)".to_string(),
            xl: "0 20px 25px -5px rgb(0 0 0 / 0.1), 0 8px 10px -6px rgb(0 0 0 / 0.1)".to_string(),
            xl2: "0 25px 50px -12px rgb(0 0 0 / 0.25)".to_string(),
            inner: "inset 0 2px 4px 0 rgb(0 0 0 / 0.05)".to_string(),
            none: "0 0 #0000".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorderWidths {
    pub n0: String,
    pub n0_5: String,
    pub n1: String,
    pub n1_5: String,
    pub n2: String,
    pub n2_5: String,
    pub n3: String,
    pub n4: String,
    pub n6: String,
    pub n8: String,
}

impl Default for BorderWidths {
    fn default() -> Self {
        Self {
            n0: "0px".to_string(),
            n0_5: "0.5px".to_string(),
            n1: "1px".to_string(),
            n1_5: "1.5px".to_string(),
            n2: "2px".to_string(),
            n2_5: "2.5px".to_string(),
            n3: "3px".to_string(),
            n4: "4px".to_string(),
            n6: "6px".to_string(),
            n8: "8px".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorderRadius {
    pub none: String,
    pub sm: String,
    pub normal: String,
    pub md: String,
    pub lg: String,
    pub xl: String,
    pub xl2: String,
    pub xl3: String,
    pub full: String,
}

impl Default for BorderRadius {
    fn default() -> Self {
        Self {
            none: "0px".to_string(),
            sm: "0.125rem".to_string(),
            normal: "0.25rem".to_string(),
            md: "0.375rem".to_string(),
            lg: "0.5rem".to_string(),
            xl: "0.75rem".to_string(),
            xl2: "1rem".to_string(),
            xl3: "1.5rem".to_string(),
            full: "9999px".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationDurations {
    pub n75: String,
    pub n100: String,
    pub n150: String,
    pub n200: String,
    pub n300: String,
    pub n500: String,
    pub n700: String,
    pub n1000: String,
}

impl Default for AnimationDurations {
    fn default() -> Self {
        Self {
            n75: "75ms".to_string(),
            n100: "100ms".to_string(),
            n150: "150ms".to_string(),
            n200: "200ms".to_string(),
            n300: "300ms".to_string(),
            n500: "500ms".to_string(),
            n700: "700ms".to_string(),
            n1000: "1000ms".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationDelays {
    pub n75: String,
    pub n100: String,
    pub n150: String,
    pub n200: String,
    pub n300: String,
    pub n500: String,
    pub n700: String,
    pub n1000: String,
}

impl Default for AnimationDelays {
    fn default() -> Self {
        Self {
            n75: "75ms".to_string(),
            n100: "100ms".to_string(),
            n150: "150ms".to_string(),
            n200: "200ms".to_string(),
            n300: "300ms".to_string(),
            n500: "500ms".to_string(),
            n700: "700ms".to_string(),
            n1000: "1000ms".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponsiveBreakpoints {
    pub sm: String,
    pub md: String,
    pub lg: String,
    pub xl: String,
    pub xl2: String,
}

impl Default for ResponsiveBreakpoints {
    fn default() -> Self {
        Self {
            sm: "640px".to_string(),
            md: "768px".to_string(),
            lg: "1024px".to_string(),
            xl: "1280px".to_string(),
            xl2: "1536px".to_string(),
        }
    }
}
