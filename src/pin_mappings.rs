#[cfg(feature = "device-selected")]
use crate::gpio::gpioa::*;
#[cfg(feature = "device-selected")]
use crate::gpio::gpiob::*;
#[allow(unused)]
#[cfg(feature = "device-selected")]
use crate::gpio::gpioc::*;
#[cfg(feature = "stm32f030xc")]
use crate::gpio::gpiod::*;
#[allow(unused)]
#[cfg(feature = "device-selected")]
use crate::gpio::gpiof::*;
#[allow(unused)]
use crate::gpio::{Alternate, AF0, AF1, AF2, AF4, AF5};
use crate::i2c::*;
use crate::serial::*;
use crate::spi::*;
#[cfg(feature = "device-selected")]
use crate::stm32::*;

macro_rules! pins {
    ($($PIN:ident => {
        $($AF:ty: $TRAIT:ty),+
    }),+) => {
        $(
            $(
                impl $TRAIT for $PIN<Alternate<$AF>> {}
            )+
        )+
    }
}

#[cfg(feature = "device-selected")]
pins! {
    PA5 => {AF0: SckPin<SPI1>},
    PA6 => {AF0: MisoPin<SPI1>},
    PA7 => {AF0: MosiPin<SPI1>},
    PA9 => {AF1: TxPin<USART1>},
    PA10 => {AF1: RxPin<USART1>},
    PB3 => {AF0: SckPin<SPI1>},
    PB4 => {AF0: MisoPin<SPI1>},
    PB5 => {AF0: MosiPin<SPI1>},
    PB6 => {
        AF0: TxPin<USART1>,
        AF1: SclPin<I2C1>
    },
    PB7 => {
        AF0: RxPin<USART1>,
        AF1: SdaPin<I2C1>
    },
    PB8 => {AF1: SclPin<I2C1>},
    PB9 => {AF1: SdaPin<I2C1>}
}

#[cfg(any(feature = "stm32f030", feature = "stm32f042"))]
pins! {
    PA11 => {AF5: SclPin<I2C1>},
    PA12 => {AF5: SdaPin<I2C1>}
}

#[cfg(feature = "stm32f030x6")]
pins! {
    PA2 => {AF1: TxPin<USART1>},
    PA3 => {AF1: RxPin<USART1>},
    PA14 => {AF1: TxPin<USART1>},
    PA15 => {AF1: RxPin<USART1>},
    PB13 => {AF0: SckPin<SPI1>},
    PB14 => {AF0: MisoPin<SPI1>},
    PB15 => {AF0: MosiPin<SPI1>}
}

#[cfg(any(
    feature = "stm32f030x8",
    feature = "stm32f030xc",
    feature = "stm32f042",
    feature = "stm32f070",
))]
pins! {
    PA2 => {AF1: TxPin<USART2>},
    PA3 => {AF1: RxPin<USART2>},
    PA14 => {AF1: TxPin<USART2>},
    PA15 => {AF1: RxPin<USART2>}
}

#[cfg(any(feature = "stm32f030xc", feature = "stm32f070xb"))]
pins! {
    PA0 => {AF4: TxPin<USART4>},
    PA1 => {AF4: RxPin<USART4>},
    PB10 => {
        AF4: TxPin<USART3>,
        AF5: SckPin<SPI2>
    },
    PB11 => {AF4: RxPin<USART3>},
    PC2 => {AF1: MisoPin<SPI2>},
    PC3 => {AF1: MosiPin<SPI2>},
    PC4 => {AF1: TxPin<USART3>},
    PC5 => {AF1: RxPin<USART3>},
    PC10 => {
        AF0: TxPin<USART4>,
        AF1: TxPin<USART3>
    },
    PC11 => {
        AF0: RxPin<USART4>,
        AF1: RxPin<USART3>
    }
}

#[cfg(feature = "stm32f030xc")]
pins! {
    PA4 => {AF5: TxPin<USART6>},
    PA5 => {AF5: RxPin<USART6>},
    PB3 => {AF4: TxPin<USART5>},
    PB4 => {AF4: RxPin<USART5>},
    PC0 => {AF2: TxPin<USART6>},
    PC1 => {AF2: RxPin<USART6>},
    PC12 => {AF2: RxPin<USART5>},
    PD2 => {AF2: TxPin<USART5>}
}

#[cfg(any(
    feature = "stm32f030x8",
    feature = "stm32f030xc",
    feature = "stm32f070xb"
))]
pins! {
    PB13 => {AF0: SckPin<SPI2>},
    PB14 => {AF0: MisoPin<SPI2>},
    PB15 => {AF0: MosiPin<SPI2>}
}

#[cfg(any(
    feature = "stm32f030x6",
    feature = "stm32f030xc",
    feature = "stm32f042",
    feature = "stm32f070x6",
))]
pins! {
    PA9 => {AF4: SclPin<I2C1>},
    PA10 => {AF4: SdaPin<I2C1>}
}

#[cfg(any(
    feature = "stm32f042",
    feature = "stm32f030x6",
    feature = "stm32f030x8",
    feature = "stm32f030xc",
    feature = "stm32f070xb"
))]
pins! {
    PB10 => {AF1: SclPin<I2C1>},
    PB11 => {AF1: SdaPin<I2C1>}
}

#[cfg(any(
    feature = "stm32f042",
    feature = "stm32f030xc",
    feature = "stm32f070x6",
))]
pins! {
    PF1 => {AF1: SclPin<I2C1>},
    PF0 => {AF1: SdaPin<I2C1>}
}

#[cfg(any(
    feature = "stm32f042",
    feature = "stm32f030xc",
    feature = "stm32f030xc",
    feature = "stm32f070xb"
))]
pins! {
    PB13 => {AF5: SclPin<I2C1>},
    PB14 => {AF5: SdaPin<I2C1>}
}
