use super::button::simple_button::{Polarity, SimpleButton};
use super::led::simple_led::SimpleLed;
use stm32f3xx_hal::gpio::{Alternate, Gpioa, Gpioe, Input, Output, Pin, PushPull, U};
use stm32f3xx_hal::prelude::*;
use stm32f3xx_hal::rcc::Clocks;
use stm32f3xx_hal::spi::{config::Config, Spi};
use stm32f3xx_hal::time::rate::{Generic, Kilohertz, Megahertz};
pub type NorthEastLed = SimpleLed<Pin<Gpioe, U<8>, Output<PushPull>>>;
pub type NorthLed = SimpleLed<Pin<Gpioe, U<9>, Output<PushPull>>>;
pub type NorthWestLed = SimpleLed<Pin<Gpioe, U<10>, Output<PushPull>>>;
pub type WestLed = SimpleLed<Pin<Gpioe, U<11>, Output<PushPull>>>;
pub type SouthWestLed = SimpleLed<Pin<Gpioe, U<12>, Output<PushPull>>>;
pub type SouthLed = SimpleLed<Pin<Gpioe, U<13>, Output<PushPull>>>;
pub type SouthEastLed = SimpleLed<Pin<Gpioe, U<14>, Output<PushPull>>>;
pub type EastLed = SimpleLed<Pin<Gpioe, U<15>, Output<PushPull>>>;
pub type GyroCs = Pin<Gpioe, U<3>, Output<PushPull>>;
pub type UserButPin = Pin<Gpioa, U<0>, Input>;
pub type UserButton = SimpleButton<UserButPin>;
pub type GyroSpi = Spi<
    stm32f3xx_hal::pac::SPI1,
    (
        Pin<Gpioa, U<5>, Alternate<PushPull, 5>>,
        Pin<Gpioa, U<6>, Alternate<PushPull, 5>>,
        Pin<Gpioa, U<7>, Alternate<PushPull, 5>>,
    ),
    u8,
>;

pub struct Board {
    pub northeast_led: NorthEastLed,
    pub north_led: NorthLed,
    pub northwest_led: NorthWestLed,
    pub west_led: WestLed,
    pub southwest_led: SouthWestLed,
    pub south_led: SouthLed,
    pub southeast_led: SouthEastLed,
    pub east_led: EastLed,
    pub user_button: UserButton,
    pub gyro_spi: GyroSpi,
    pub gyro_cs: GyroCs,
    pub clocks: Clocks,
}

impl Board {
    pub fn new(p: stm32f3xx_hal::pac::Peripherals) -> Self {
        let mut rcc = p.RCC.constrain();
        let mut gpioa = p.GPIOA.split(&mut rcc.ahb);
        let mut gpioe = p.GPIOE.split(&mut rcc.ahb);

        let pe8 = gpioe
            .pe8
            .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
        let pe9 = gpioe
            .pe9
            .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
        let pe10 = gpioe
            .pe10
            .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
        let pe11 = gpioe
            .pe11
            .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
        let pe12 = gpioe
            .pe12
            .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
        let pe13 = gpioe
            .pe13
            .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
        let pe14 = gpioe
            .pe14
            .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
        let pe15 = gpioe
            .pe15
            .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
        let pe3 = gpioe
            .pe3
            .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);

        let pa0 = gpioa.pa0.into_input(&mut gpioa.moder);

        let northeast_led: NorthEastLed = SimpleLed::new(pe8);
        let north_led: NorthLed = SimpleLed::new(pe9);
        let northwest_led: NorthWestLed = SimpleLed::new(pe10);
        let west_led: WestLed = SimpleLed::new(pe11);
        let southwest_led: SouthWestLed = SimpleLed::new(pe12);
        let south_led: SouthLed = SimpleLed::new(pe13);
        let southeast_led: SouthEastLed = SimpleLed::new(pe14);
        let east_led: EastLed = SimpleLed::new(pe15);
        let user_button = SimpleButton::new(pa0, Polarity::ActiveHigh);

        // ----------- GYRO SPI -------------------
        let sck =
            gpioa
                .pa5
                .into_af_push_pull::<5>(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);
        let miso =
            gpioa
                .pa6
                .into_af_push_pull::<5>(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);
        let mosi =
            gpioa
                .pa7
                .into_af_push_pull::<5>(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);
        let mut f = p.FLASH.constrain();
        let r = rcc.cfgr.sysclk(48.MHz()).freeze(&mut f.acr);

        let config = Config::default().frequency(Kilohertz::new(1));

        let spi: Spi<_, _, u8> = Spi::new(p.SPI1, (sck, miso, mosi), config, r, &mut rcc.apb2);

        let ba = Board {
            northeast_led,
            north_led,
            northwest_led,
            west_led,
            southwest_led,
            south_led,
            southeast_led,
            east_led,
            user_button,
            gyro_spi: spi,
            gyro_cs: pe3,
            clocks: r,
        };

        ba
    }
}
