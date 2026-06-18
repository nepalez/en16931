//! The package type code (`BT-130` unit position, UN/ECE Recommendation 21).
//!
//! Source: [EN-16931 codelist](https://github.com/ConnectingEurope/eInvoicing-EN16931/blob/master/ubl/schematron/codelist/EN16931-UBL-codes.sch), rule `BR-CL-23`.
//! Names follow [UN/ECE Recommendation 21](https://unece.org/trade/uncefact/cl-recommendations).
//! The set is the Recommendation 21 extension of the unit code list, prefixed with `X`.

use crate::Error;
use crate::prelude::*;

/// The package type code (UN/ECE Rec 21): the kind of package a quantity is expressed in.
/// It mirrors the shape of the reused unit type — `from_code`, `code`, `name`, and `ALL`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PackageCode {
    /// X1.
    X1,
    /// Drum, steel.
    DrumSteel,
    /// Drum, aluminium.
    DrumAluminium,
    /// Drum, plywood.
    DrumPlywood,
    /// Container, flexible.
    ContainerFlexible,
    /// Drum, fibre.
    DrumFibre,
    /// Drum, wooden.
    DrumWooden,
    /// Barrel, wooden.
    BarrelWooden,
    /// Jerrican, steel.
    JerricanSteel,
    /// Jerrican, plastic.
    JerricanPlastic,
    /// Bag, super bulk.
    BagSuperBulk,
    /// Bag, polybag.
    BagPolybag,
    /// Box, steel.
    BoxSteel,
    /// Box, aluminium.
    BoxAluminium,
    /// Box, natural wood.
    BoxNaturalWood,
    /// Box, plywood.
    BoxPlywood,
    /// Box, reconstituted wood.
    BoxReconstitutedWood,
    /// Box, fibreboard.
    BoxFibreboard,
    /// Box, plastic.
    BoxPlastic,
    /// Bag, woven plastic.
    BagWovenPlastic,
    /// Bag, textile.
    BagTextile,
    /// Bag, paper.
    BagPaper,
    /// Composite packaging, plastic receptacle.
    CompositePackagingPlasticReceptacle,
    /// Composite packaging, glass receptacle.
    CompositePackagingGlassReceptacle,
    /// Case, car.
    CaseCar,
    /// Case, wooden.
    CaseWooden,
    /// Pallet, wooden.
    PalletWooden,
    /// Crate, wooden.
    CrateWooden,
    /// Bundle, wooden.
    BundleWooden,
    /// Intermediate bulk container, rigid plastic.
    IntermediateBulkContainerRigidPlastic,
    /// Receptacle, fibre.
    ReceptacleFibre,
    /// Receptacle, paper.
    ReceptaclePaper,
    /// Receptacle, wooden.
    ReceptacleWooden,
    /// Aerosol.
    Aerosol,
    /// Pallet, modular, collars 80cms * 60cms.
    PalletModularCollars80cms60cms,
    /// Pallet, shrinkwrapped.
    PalletShrinkwrapped,
    /// Pallet, 100cms * 110cms.
    Pallet100cms110cms,
    /// Clamshell.
    Clamshell,
    /// Cone.
    Cone,
    /// Ball.
    Ball,
    /// Ampoule, non-protected.
    AmpouleNonProtected,
    /// Ampoule, protected.
    AmpouleProtected,
    /// Atomizer.
    Atomizer,
    /// Capsule.
    Capsule,
    /// Belt.
    Belt,
    /// Barrel.
    Barrel,
    /// Bobbin.
    Bobbin,
    /// Bottlecrate / bottlerack.
    BottlecrateBottlerack,
    /// Board.
    Board,
    /// Bundle.
    Bundle,
    /// Balloon, non-protected.
    BalloonNonProtected,
    /// Bag.
    Bag,
    /// Bunch.
    Bunch,
    /// Bin.
    Bin,
    /// Bucket.
    Bucket,
    /// Basket.
    Basket,
    /// Bale, compressed.
    BaleCompressed,
    /// Basin.
    Basin,
    /// Bale, non-compressed.
    BaleNonCompressed,
    /// Bottle, non-protected, cylindrical.
    BottleNonProtectedCylindrical,
    /// Balloon, protected.
    BalloonProtected,
    /// Bottle, protected cylindrical.
    BottleProtectedCylindrical,
    /// Bar.
    Bar,
    /// Bottle, non-protected, bulbous.
    BottleNonProtectedBulbous,
    /// Bolt.
    Bolt,
    /// Butt.
    Butt,
    /// Bottle, protected bulbous.
    BottleProtectedBulbous,
    /// Box, for liquids.
    BoxForLiquids,
    /// Box.
    Box,
    /// Board, in bundle/bunch/truss.
    BoardInBundleBunchTruss,
    /// Bars, in bundle/bunch/truss.
    BarsInBundleBunchTruss,
    /// Can, rectangular.
    CanRectangular,
    /// Crate, beer.
    CrateBeer,
    /// Churn.
    Churn,
    /// Can, with handle and spout.
    CanWithHandleAndSpout,
    /// Creel.
    Creel,
    /// Coffer.
    Coffer,
    /// Cage.
    Cage,
    /// Chest.
    Chest,
    /// Canister.
    Canister,
    /// Coffin.
    Coffin,
    /// Cask.
    Cask,
    /// Coil.
    Coil,
    /// Card.
    Card,
    /// Container, not otherwise specified as transport equipment.
    ContainerNotOtherwiseSpecifiedAsTransportEquipment,
    /// Carboy, non-protected.
    CarboyNonProtected,
    /// Carboy, protected.
    CarboyProtected,
    /// Cartridge.
    Cartridge,
    /// Crate.
    Crate,
    /// Case.
    Case,
    /// Carton.
    Carton,
    /// Cup.
    Cup,
    /// Cover.
    Cover,
    /// Cage, roll.
    CageRoll,
    /// Can, cylindrical.
    CanCylindrical,
    /// Cylinder.
    Cylinder,
    /// Canvas.
    Canvas,
    /// Crate, multiple layer, plastic.
    CrateMultipleLayerPlastic,
    /// Crate, multiple layer, wooden.
    CrateMultipleLayerWooden,
    /// Crate, multiple layer, cardboard.
    CrateMultipleLayerCardboard,
    /// Cage, Commonwealth Handling Equipment Pool  (CHEP).
    CageCommonwealthHandlingEquipmentPoolCHEP,
    /// Box, Commonwealth Handling Equipment Pool (CHEP), Eurobox.
    BoxCommonwealthHandlingEquipmentPoolCHEPEurobox,
    /// Drum, iron.
    DrumIron,
    /// Demijohn, non-protected.
    DemijohnNonProtected,
    /// Crate, bulk, cardboard.
    CrateBulkCardboard,
    /// Crate, bulk, plastic.
    CrateBulkPlastic,
    /// Crate, bulk, wooden.
    CrateBulkWooden,
    /// Dispenser.
    Dispenser,
    /// Demijohn, protected.
    DemijohnProtected,
    /// Drum.
    Drum,
    /// Tray, one layer no cover, plastic.
    TrayOneLayerNoCoverPlastic,
    /// Tray, one layer no cover, wooden.
    TrayOneLayerNoCoverWooden,
    /// Tray, one layer no cover, polystyrene.
    TrayOneLayerNoCoverPolystyrene,
    /// Tray, one layer no cover, cardboard.
    TrayOneLayerNoCoverCardboard,
    /// Tray, two layers no cover, plastic tray.
    TrayTwoLayersNoCoverPlasticTray,
    /// Tray, two layers no cover, wooden.
    TrayTwoLayersNoCoverWooden,
    /// Tray, two layers no cover, cardboard.
    TrayTwoLayersNoCoverCardboard,
    /// Bag, plastic.
    BagPlastic,
    /// Case, with pallet base.
    CaseWithPalletBase,
    /// Case, with pallet base, wooden.
    CaseWithPalletBaseWooden,
    /// Case, with pallet base, cardboard.
    CaseWithPalletBaseCardboard,
    /// Case, with pallet base, plastic.
    CaseWithPalletBasePlastic,
    /// Case, with pallet base, metal.
    CaseWithPalletBaseMetal,
    /// Case, isothermic.
    CaseIsothermic,
    /// Envelope.
    Envelope,
    /// Flexibag.
    Flexibag,
    /// Crate, fruit.
    CrateFruit,
    /// Crate, framed.
    CrateFramed,
    /// Flexitank.
    Flexitank,
    /// Firkin.
    Firkin,
    /// Flask.
    Flask,
    /// Footlocker.
    Footlocker,
    /// Filmpack.
    Filmpack,
    /// Frame.
    Frame,
    /// Foodtainer.
    Foodtainer,
    /// Cart, flatbed.
    CartFlatbed,
    /// Bag, flexible container.
    BagFlexibleContainer,
    /// Bottle, gas.
    BottleGas,
    /// Girder.
    Girder,
    /// Container, gallon.
    ContainerGallon,
    /// Receptacle, glass.
    ReceptacleGlass,
    /// Tray, containing horizontally stacked flat items.
    TrayContainingHorizontallyStackedFlatItems,
    /// Bag, gunny.
    BagGunny,
    /// Girders, in bundle/bunch/truss.
    GirdersInBundleBunchTruss,
    /// Basket, with handle, plastic.
    BasketWithHandlePlastic,
    /// Basket, with handle, wooden.
    BasketWithHandleWooden,
    /// Basket, with handle, cardboard.
    BasketWithHandleCardboard,
    /// Hogshead.
    Hogshead,
    /// Hanger.
    Hanger,
    /// Hamper.
    Hamper,
    /// Package, display, wooden.
    PackageDisplayWooden,
    /// Package, display, cardboard.
    PackageDisplayCardboard,
    /// Package, display, plastic.
    PackageDisplayPlastic,
    /// Package, display, metal.
    PackageDisplayMetal,
    /// Package, show.
    PackageShow,
    /// Package, flow.
    PackageFlow,
    /// Package, paper wrapped.
    PackagePaperWrapped,
    /// Drum, plastic.
    DrumPlastic,
    /// Package, cardboard, with bottle grip-holes.
    PackageCardboardWithBottleGripHoles,
    /// Tray, rigid, lidded stackable (CEN TS 14482:2002).
    TrayRigidLiddedStackableCENTS144822002,
    /// Ingot.
    Ingot,
    /// Ingots, in bundle/bunch/truss.
    IngotsInBundleBunchTruss,
    /// Bag, jumbo.
    BagJumbo,
    /// Jerrican, rectangular.
    JerricanRectangular,
    /// Jug.
    Jug,
    /// Jar.
    Jar,
    /// Jutebag.
    Jutebag,
    /// Jerrican, cylindrical.
    JerricanCylindrical,
    /// Keg.
    Keg,
    /// Kit.
    Kit,
    /// Luggage.
    Luggage,
    /// Log.
    Log,
    /// Lot.
    Lot,
    /// Lug.
    Lug,
    /// Liftvan.
    Liftvan,
    /// Logs, in bundle/bunch/truss.
    LogsInBundleBunchTruss,
    /// Crate, metal.
    CrateMetal,
    /// Bag, multiply.
    BagMultiply,
    /// Crate, milk.
    CrateMilk,
    /// Container, metal.
    ContainerMetal,
    /// Receptacle, metal.
    ReceptacleMetal,
    /// Sack, multi-wall.
    SackMultiWall,
    /// Mat.
    Mat,
    /// Receptacle, plastic wrapped.
    ReceptaclePlasticWrapped,
    /// Matchbox.
    Matchbox,
    /// Not available.
    NotAvailable,
    /// Unpacked or unpackaged.
    UnpackedOrUnpackaged,
    /// Unpacked or unpackaged, single unit.
    UnpackedOrUnpackagedSingleUnit,
    /// Unpacked or unpackaged, multiple units.
    UnpackedOrUnpackagedMultipleUnits,
    /// Nest.
    Nest,
    /// Net.
    Net,
    /// Net, tube, plastic.
    NetTubePlastic,
    /// Net, tube, textile.
    NetTubeTextile,
    /// Two-sided cage on wheels with fixing strap.
    TwoSidedCageOnWheelsWithFixingStrap,
    /// Trolley.
    Trolley,
    /// Oneway pallet ISO 0 - 1/2 EURO Pallet.
    OnewayPalletISO012EUROPallet,
    /// Oneway pallet ISO 1 - 1/1 EURO Pallet.
    OnewayPalletISO111EUROPallet,
    /// Oneway pallet ISO 2 - 2/1 EURO Pallet.
    OnewayPalletISO221EUROPallet,
    /// Pallet with exceptional dimensions.
    PalletWithExceptionalDimensions,
    /// Wooden pallet  40 cm x 80 cm.
    WoodenPallet40CmX80Cm,
    /// Plastic pallet SRS 60 cm x 80 cm.
    PlasticPalletSRS60CmX80Cm,
    /// Plastic pallet SRS 80 cm x 120 cm.
    PlasticPalletSRS80CmX120Cm,
    /// Pallet, CHEP 40 cm x 60 cm.
    PalletCHEP40CmX60Cm,
    /// Pallet, CHEP 80 cm x 120 cm.
    PalletCHEP80CmX120Cm,
    /// Pallet, CHEP 100 cm x 120 cm.
    PalletCHEP100CmX120Cm,
    /// Pallet, AS 4068-1993.
    PalletAS40681993,
    /// Pallet, ISO T11.
    PalletISOT11,
    /// Platform, unspecified weight or dimension.
    PlatformUnspecifiedWeightOrDimension,
    /// Pallet ISO 0 - 1/2 EURO Pallet.
    PalletISO012EUROPallet,
    /// Pallet ISO 1 - 1/1 EURO Pallet.
    PalletISO111EUROPallet,
    /// Pallet ISO 2 – 2/1 EURO Pallet.
    PalletISO221EUROPallet,
    /// 1/4 EURO Pallet.
    P14EUROPallet,
    /// Block.
    Block,
    /// 1/8 EURO Pallet.
    P18EUROPallet,
    /// Synthetic pallet ISO 1.
    SyntheticPalletISO1,
    /// Synthetic pallet ISO 2.
    SyntheticPalletISO2,
    /// Wholesaler pallet.
    WholesalerPallet,
    /// Pallet 80 X 100 cm.
    Pallet80X100Cm,
    /// Pallet 60 X 100 cm.
    Pallet60X100Cm,
    /// Oneway pallet.
    OnewayPallet,
    /// Octabin.
    Octabin,
    /// Container, outer.
    ContainerOuter,
    /// Returnable pallet.
    ReturnablePallet,
    /// Large bag, pallet sized.
    LargeBagPalletSized,
    /// A wheeled pallet with raised rim (81 x 67 x 135).
    AWheeledPalletWithRaisedRim81X67X135,
    /// A Wheeled pallet with raised rim (81 x 72 x 135).
    AWheeledPalletWithRaisedRim81X72X135,
    /// Wheeled pallet with raised rim ( 81 x 60 x 16).
    WheeledPalletWithRaisedRim81X60X16,
    /// CHEP pallet 60 cm x 80 cm.
    CHEPPallet60CmX80Cm,
    /// Pan.
    Pan,
    /// LPR pallet 60 cm x 80 cm.
    LPRPallet60CmX80Cm,
    /// LPR pallet 80 cm x 120 cm.
    LPRPallet80CmX120Cm,
    /// Packet.
    Packet,
    /// Pallet, box Combined open-ended box and pallet.
    PalletBoxCombinedOpenEndedBoxAndPallet,
    /// Parcel.
    Parcel,
    /// Pallet, modular, collars 80cms * 100cms.
    PalletModularCollars80cms100cms,
    /// Pallet, modular, collars 80cms * 120cms.
    PalletModularCollars80cms120cms,
    /// Pen.
    Pen,
    /// Plate.
    Plate,
    /// Pitcher.
    Pitcher,
    /// Pipe.
    Pipe,
    /// Punnet.
    Punnet,
    /// Package.
    Package,
    /// Pail.
    Pail,
    /// Plank.
    Plank,
    /// Pouch.
    Pouch,
    /// Piece.
    Piece,
    /// Receptacle, plastic.
    ReceptaclePlastic,
    /// Pot.
    Pot,
    /// Tray.
    Tray,
    /// Pipes, in bundle/bunch/truss.
    PipesInBundleBunchTruss,
    /// Pallet.
    Pallet,
    /// Plates, in bundle/bunch/truss.
    PlatesInBundleBunchTruss,
    /// Planks, in bundle/bunch/truss.
    PlanksInBundleBunchTruss,
    /// Drum, steel, non-removable head.
    DrumSteelNonRemovableHead,
    /// Drum, steel, removable head.
    DrumSteelRemovableHead,
    /// Drum, aluminum, non-removable head.
    DrumAluminiumNonRemovableHead,
    /// Drum, aluminum, removable head.
    DrumAluminiumRemovableHead,
    /// Drum, plastic, non-removable head.
    DrumPlasticNonRemovableHead,
    /// Drum, plastic, removable head.
    DrumPlasticRemovableHead,
    /// Barrel, wooden, bung type.
    BarrelWoodenBungType,
    /// Barrel, wooden, removable head.
    BarrelWoodenRemovableHead,
    /// Jerrican, steel, non-removable head.
    JerricanSteelNonRemovableHead,
    /// Jerrican, steel, removable head.
    JerricanSteelRemovableHead,
    /// Jerrican, plastic, non-removable head.
    JerricanPlasticNonRemovableHead,
    /// Jerrican, plastic, removable head.
    JerricanPlasticRemovableHead,
    /// Box, wooden, natural wood, ordinary.
    BoxWoodenNaturalWoodOrdinary,
    /// Box, wooden, natural wood, with sift proof walls.
    BoxWoodenNaturalWoodWithSiftProofWalls,
    /// Box, plastic, expanded.
    BoxPlasticExpanded,
    /// Box, plastic, solid.
    BoxPlasticSolid,
    /// Rod.
    Rod,
    /// Ring.
    Ring,
    /// Rack, clothing hanger.
    RackClothingHanger,
    /// Rack.
    Rack,
    /// Reel.
    Reel,
    /// Roll.
    Roll,
    /// Rednet.
    Rednet,
    /// Rods, in bundle/bunch/truss.
    RodsInBundleBunchTruss,
    /// Sack.
    Sack,
    /// Slab.
    Slab,
    /// Crate, shallow.
    CrateShallow,
    /// Spindle.
    Spindle,
    /// Sea-chest.
    SeaChest,
    /// Sachet.
    Sachet,
    /// Skid.
    Skid,
    /// Case, skeleton.
    CaseSkeleton,
    /// Slipsheet.
    Slipsheet,
    /// Sheetmetal.
    Sheetmetal,
    /// Spool.
    Spool,
    /// Sheet, plastic wrapping.
    SheetPlasticWrapping,
    /// Case, steel.
    CaseSteel,
    /// Sheet.
    Sheet,
    /// Suitcase.
    Suitcase,
    /// Envelope, steel.
    EnvelopeSteel,
    /// Shrinkwrapped.
    Shrinkwrapped,
    /// Set.
    Set,
    /// Sleeve.
    Sleeve,
    /// Sheets, in bundle/bunch/truss.
    SheetsInBundleBunchTruss,
    /// Tablet.
    Tablet,
    /// Tub.
    Tub,
    /// Tea-chest.
    TeaChest,
    /// Tube, collapsible.
    TubeCollapsible,
    /// Tyre.
    Tyre,
    /// Tank container, generic.
    TankContainerGeneric,
    /// Tierce.
    Tierce,
    /// Tank, rectangular.
    TankRectangular,
    /// Tub, with lid.
    TubWithLid,
    /// Tin.
    Tin,
    /// Tun.
    Tun,
    /// Trunk.
    Trunk,
    /// Truss.
    Truss,
    /// Bag, tote.
    BagTote,
    /// Tube.
    Tube,
    /// Tube, with nozzle.
    TubeWithNozzle,
    /// Pallet, triwall.
    PalletTriwall,
    /// Tank, cylindrical.
    TankCylindrical,
    /// Tubes, in bundle/bunch/truss.
    TubesInBundleBunchTruss,
    /// Uncaged.
    Uncaged,
    /// Unit.
    Unit,
    /// Vat.
    Vat,
    /// Bulk, gas (at 1031 mbar and 15°C).
    BulkGasAt1031MbarAnd15C,
    /// Vial.
    Vial,
    /// Vanpack.
    Vanpack,
    /// Bulk, liquid.
    BulkLiquid,
    /// Vehicle.
    Vehicle,
    /// Bulk, solid, large particles (“nodules”).
    BulkSolidLargeParticlesNodules,
    /// Vacuum-packed.
    VacuumPacked,
    /// Bulk, liquefied gas (at abnormal temperature/pressure).
    BulkLiquefiedGasAtAbnormalTemperaturePressure,
    /// Bulk, solid, granular particles (“grains”).
    BulkSolidGranularParticlesGrains,
    /// Bulk, scrap metal.
    BulkScrapMetal,
    /// Bulk, solid, fine particles (“powders”).
    BulkSolidFineParticlesPowders,
    /// Intermediate bulk container.
    IntermediateBulkContainer,
    /// Wickerbottle.
    Wickerbottle,
    /// Intermediate bulk container, steel.
    IntermediateBulkContainerSteel,
    /// Intermediate bulk container, aluminium.
    IntermediateBulkContainerAluminium,
    /// Intermediate bulk container, metal.
    IntermediateBulkContainerMetal,
    /// Intermediate bulk container, steel, pressurised > 10 kpa.
    IntermediateBulkContainerSteelPressurised10Kpa,
    /// Intermediate bulk container, aluminium, pressurised > 10 kpa.
    IntermediateBulkContainerAluminiumPressurised10Kpa,
    /// Intermediate bulk container, metal, pressure 10 kpa.
    IntermediateBulkContainerMetalPressure10Kpa,
    /// Intermediate bulk container, steel, liquid.
    IntermediateBulkContainerSteelLiquid,
    /// Intermediate bulk container, aluminium, liquid.
    IntermediateBulkContainerAluminiumLiquid,
    /// Intermediate bulk container, metal, liquid.
    IntermediateBulkContainerMetalLiquid,
    /// Intermediate bulk container, woven plastic, without coat/liner.
    IntermediateBulkContainerWovenPlasticWithoutCoatLiner,
    /// Intermediate bulk container, woven plastic, coated.
    IntermediateBulkContainerWovenPlasticCoated,
    /// Intermediate bulk container, woven plastic, with liner.
    IntermediateBulkContainerWovenPlasticWithLiner,
    /// Intermediate bulk container, woven plastic, coated and liner.
    IntermediateBulkContainerWovenPlasticCoatedAndLiner,
    /// Intermediate bulk container, plastic film.
    IntermediateBulkContainerPlasticFilm,
    /// Intermediate bulk container, textile without coat/liner.
    IntermediateBulkContainerTextileWithOutCoatLiner,
    /// Intermediate bulk container, natural wood, with inner liner.
    IntermediateBulkContainerNaturalWoodWithInnerLiner,
    /// Intermediate bulk container, textile, coated.
    IntermediateBulkContainerTextileCoated,
    /// Intermediate bulk container, textile, with liner.
    IntermediateBulkContainerTextileWithLiner,
    /// Intermediate bulk container, textile, coated and liner.
    IntermediateBulkContainerTextileCoatedAndLiner,
    /// Intermediate bulk container, plywood, with inner liner.
    IntermediateBulkContainerPlywoodWithInnerLiner,
    /// Intermediate bulk container, reconstituted wood, with inner liner.
    IntermediateBulkContainerReconstitutedWoodWithInnerLiner,
    /// Bag, woven plastic, without inner coat/liner.
    BagWovenPlasticWithoutInnerCoatLiner,
    /// Bag, woven plastic, sift proof.
    BagWovenPlasticSiftProof,
    /// Bag, woven plastic, water-resistant.
    BagWovenPlasticWaterResistant,
    /// Bag, plastics film.
    BagPlasticsFilm,
    /// Bag, textile, without inner coat/liner.
    BagTextileWithoutInnerCoatLiner,
    /// Bag, textile, sift proof.
    BagTextileSiftProof,
    /// Bag, textile, water-resistant.
    BagTextileWaterResistant,
    /// Bag, paper, multi-wall.
    BagPaperMultiWall,
    /// Bag, paper, multi-wall, water-resistant.
    BagPaperMultiWallWaterResistant,
    /// Composite packaging, plastic receptacle in steel drum.
    CompositePackagingPlasticReceptacleInSteelDrum,
    /// Composite packaging, plastic receptacle in steel crate box.
    CompositePackagingPlasticReceptacleInSteelCrateBox,
    /// Composite packaging, plastic receptacle in aluminium drum.
    CompositePackagingPlasticReceptacleInAluminiumDrum,
    /// Composite packaging, plastic receptacle in aluminium crate.
    CompositePackagingPlasticReceptacleInAluminiumCrate,
    /// Composite packaging, plastic receptacle in wooden box.
    CompositePackagingPlasticReceptacleInWoodenBox,
    /// Composite packaging, plastic receptacle in plywood drum.
    CompositePackagingPlasticReceptacleInPlywoodDrum,
    /// Composite packaging, plastic receptacle in plywood box.
    CompositePackagingPlasticReceptacleInPlywoodBox,
    /// Composite packaging, plastic receptacle in fiber drum.
    CompositePackagingPlasticReceptacleInFibreDrum,
    /// Composite packaging, plastic receptacle in fiberboard box.
    CompositePackagingPlasticReceptacleInFibreboardBox,
    /// Composite packaging, plastic receptacle in plastic drum.
    CompositePackagingPlasticReceptacleInPlasticDrum,
    /// Composite packaging, plastic receptacle in solid plastic box.
    CompositePackagingPlasticReceptacleInSolidPlasticBox,
    /// Composite packaging, glass receptacle in steel drum.
    CompositePackagingGlassReceptacleInSteelDrum,
    /// Composite packaging, glass receptacle in steel crate box.
    CompositePackagingGlassReceptacleInSteelCrateBox,
    /// Composite packaging, glass receptacle in aluminum drum.
    CompositePackagingGlassReceptacleInAluminiumDrum,
    /// Composite packaging, glass receptacle in aluminum crate.
    CompositePackagingGlassReceptacleInAluminiumCrate,
    /// Composite packaging, glass receptacle in wooden box.
    CompositePackagingGlassReceptacleInWoodenBox,
    /// Composite packaging, glass receptacle in plywood drum.
    CompositePackagingGlassReceptacleInPlywoodDrum,
    /// Composite packaging, glass receptacle in wickerwork hamper.
    CompositePackagingGlassReceptacleInWickerworkHamper,
    /// Composite packaging, glass receptacle in fiber drum.
    CompositePackagingGlassReceptacleInFibreDrum,
    /// Composite packaging, glass receptacle in fiberboard box.
    CompositePackagingGlassReceptacleInFibreboardBox,
    /// Composite packaging, glass receptacle in expandable plastic pack.
    CompositePackagingGlassReceptacleInExpandablePlasticPack,
    /// Composite packaging, glass receptacle in solid plastic pack.
    CompositePackagingGlassReceptacleInSolidPlasticPack,
    /// Intermediate bulk container, paper, multi-wall.
    IntermediateBulkContainerPaperMultiWall,
    /// Bag, large.
    BagLarge,
    /// Intermediate bulk container, paper, multi-wall, water-resistant.
    IntermediateBulkContainerPaperMultiWallWaterResistant,
    /// Intermediate bulk container, rigid plastic, with structural equipment, solids.
    IntermediateBulkContainerRigidPlasticWithStructuralEquipmentSolids,
    /// Intermediate bulk container, rigid plastic, freestanding, solids.
    IntermediateBulkContainerRigidPlasticFreestandingSolids,
    /// Intermediate bulk container, rigid plastic, with structural equipment, pressurized.
    IntermediateBulkContainerRigidPlasticWithStructuralEquipmentPressurised,
    /// Intermediate bulk container, rigid plastic, freestanding, pressurized.
    IntermediateBulkContainerRigidPlasticFreestandingPressurised,
    /// Intermediate bulk container, rigid plastic, with structural equipment, liquids.
    IntermediateBulkContainerRigidPlasticWithStructuralEquipmentLiquids,
    /// Intermediate bulk container, rigid plastic, freestanding, liquids.
    IntermediateBulkContainerRigidPlasticFreestandingLiquids,
    /// Intermediate bulk container, composite, rigid plastic, solids.
    IntermediateBulkContainerCompositeRigidPlasticSolids,
    /// Intermediate bulk container, composite, flexible plastic, solids.
    IntermediateBulkContainerCompositeFlexiblePlasticSolids,
    /// Intermediate bulk container, composite, rigid plastic, pressurized.
    IntermediateBulkContainerCompositeRigidPlasticPressurised,
    /// Intermediate bulk container, composite, flexible plastic, pressurized.
    IntermediateBulkContainerCompositeFlexiblePlasticPressurised,
    /// Intermediate bulk container, composite, rigid plastic, liquids.
    IntermediateBulkContainerCompositeRigidPlasticLiquids,
    /// Intermediate bulk container, composite, flexible plastic, liquids.
    IntermediateBulkContainerCompositeFlexiblePlasticLiquids,
    /// Intermediate bulk container, composite.
    IntermediateBulkContainerComposite,
    /// Intermediate bulk container, fiberboard.
    IntermediateBulkContainerFibreboard,
    /// Intermediate bulk container, flexible.
    IntermediateBulkContainerFlexible,
    /// Intermediate bulk container, metal, other than steel.
    IntermediateBulkContainerMetalOtherThanSteel,
    /// Intermediate bulk container, natural wood.
    IntermediateBulkContainerNaturalWood,
    /// Intermediate bulk container, plywood.
    IntermediateBulkContainerPlywood,
    /// Intermediate bulk container, reconstituted wood.
    IntermediateBulkContainerReconstitutedWood,
    /// Mutually defined.
    MutuallyDefined,
}

impl PackageCode {
    /// Every package type code, in code order.
    pub const ALL: &'static [PackageCode] = &[
        Self::X1,
        Self::DrumSteel,
        Self::DrumAluminium,
        Self::DrumPlywood,
        Self::ContainerFlexible,
        Self::DrumFibre,
        Self::DrumWooden,
        Self::BarrelWooden,
        Self::JerricanSteel,
        Self::JerricanPlastic,
        Self::BagSuperBulk,
        Self::BagPolybag,
        Self::BoxSteel,
        Self::BoxAluminium,
        Self::BoxNaturalWood,
        Self::BoxPlywood,
        Self::BoxReconstitutedWood,
        Self::BoxFibreboard,
        Self::BoxPlastic,
        Self::BagWovenPlastic,
        Self::BagTextile,
        Self::BagPaper,
        Self::CompositePackagingPlasticReceptacle,
        Self::CompositePackagingGlassReceptacle,
        Self::CaseCar,
        Self::CaseWooden,
        Self::PalletWooden,
        Self::CrateWooden,
        Self::BundleWooden,
        Self::IntermediateBulkContainerRigidPlastic,
        Self::ReceptacleFibre,
        Self::ReceptaclePaper,
        Self::ReceptacleWooden,
        Self::Aerosol,
        Self::PalletModularCollars80cms60cms,
        Self::PalletShrinkwrapped,
        Self::Pallet100cms110cms,
        Self::Clamshell,
        Self::Cone,
        Self::Ball,
        Self::AmpouleNonProtected,
        Self::AmpouleProtected,
        Self::Atomizer,
        Self::Capsule,
        Self::Belt,
        Self::Barrel,
        Self::Bobbin,
        Self::BottlecrateBottlerack,
        Self::Board,
        Self::Bundle,
        Self::BalloonNonProtected,
        Self::Bag,
        Self::Bunch,
        Self::Bin,
        Self::Bucket,
        Self::Basket,
        Self::BaleCompressed,
        Self::Basin,
        Self::BaleNonCompressed,
        Self::BottleNonProtectedCylindrical,
        Self::BalloonProtected,
        Self::BottleProtectedCylindrical,
        Self::Bar,
        Self::BottleNonProtectedBulbous,
        Self::Bolt,
        Self::Butt,
        Self::BottleProtectedBulbous,
        Self::BoxForLiquids,
        Self::Box,
        Self::BoardInBundleBunchTruss,
        Self::BarsInBundleBunchTruss,
        Self::CanRectangular,
        Self::CrateBeer,
        Self::Churn,
        Self::CanWithHandleAndSpout,
        Self::Creel,
        Self::Coffer,
        Self::Cage,
        Self::Chest,
        Self::Canister,
        Self::Coffin,
        Self::Cask,
        Self::Coil,
        Self::Card,
        Self::ContainerNotOtherwiseSpecifiedAsTransportEquipment,
        Self::CarboyNonProtected,
        Self::CarboyProtected,
        Self::Cartridge,
        Self::Crate,
        Self::Case,
        Self::Carton,
        Self::Cup,
        Self::Cover,
        Self::CageRoll,
        Self::CanCylindrical,
        Self::Cylinder,
        Self::Canvas,
        Self::CrateMultipleLayerPlastic,
        Self::CrateMultipleLayerWooden,
        Self::CrateMultipleLayerCardboard,
        Self::CageCommonwealthHandlingEquipmentPoolCHEP,
        Self::BoxCommonwealthHandlingEquipmentPoolCHEPEurobox,
        Self::DrumIron,
        Self::DemijohnNonProtected,
        Self::CrateBulkCardboard,
        Self::CrateBulkPlastic,
        Self::CrateBulkWooden,
        Self::Dispenser,
        Self::DemijohnProtected,
        Self::Drum,
        Self::TrayOneLayerNoCoverPlastic,
        Self::TrayOneLayerNoCoverWooden,
        Self::TrayOneLayerNoCoverPolystyrene,
        Self::TrayOneLayerNoCoverCardboard,
        Self::TrayTwoLayersNoCoverPlasticTray,
        Self::TrayTwoLayersNoCoverWooden,
        Self::TrayTwoLayersNoCoverCardboard,
        Self::BagPlastic,
        Self::CaseWithPalletBase,
        Self::CaseWithPalletBaseWooden,
        Self::CaseWithPalletBaseCardboard,
        Self::CaseWithPalletBasePlastic,
        Self::CaseWithPalletBaseMetal,
        Self::CaseIsothermic,
        Self::Envelope,
        Self::Flexibag,
        Self::CrateFruit,
        Self::CrateFramed,
        Self::Flexitank,
        Self::Firkin,
        Self::Flask,
        Self::Footlocker,
        Self::Filmpack,
        Self::Frame,
        Self::Foodtainer,
        Self::CartFlatbed,
        Self::BagFlexibleContainer,
        Self::BottleGas,
        Self::Girder,
        Self::ContainerGallon,
        Self::ReceptacleGlass,
        Self::TrayContainingHorizontallyStackedFlatItems,
        Self::BagGunny,
        Self::GirdersInBundleBunchTruss,
        Self::BasketWithHandlePlastic,
        Self::BasketWithHandleWooden,
        Self::BasketWithHandleCardboard,
        Self::Hogshead,
        Self::Hanger,
        Self::Hamper,
        Self::PackageDisplayWooden,
        Self::PackageDisplayCardboard,
        Self::PackageDisplayPlastic,
        Self::PackageDisplayMetal,
        Self::PackageShow,
        Self::PackageFlow,
        Self::PackagePaperWrapped,
        Self::DrumPlastic,
        Self::PackageCardboardWithBottleGripHoles,
        Self::TrayRigidLiddedStackableCENTS144822002,
        Self::Ingot,
        Self::IngotsInBundleBunchTruss,
        Self::BagJumbo,
        Self::JerricanRectangular,
        Self::Jug,
        Self::Jar,
        Self::Jutebag,
        Self::JerricanCylindrical,
        Self::Keg,
        Self::Kit,
        Self::Luggage,
        Self::Log,
        Self::Lot,
        Self::Lug,
        Self::Liftvan,
        Self::LogsInBundleBunchTruss,
        Self::CrateMetal,
        Self::BagMultiply,
        Self::CrateMilk,
        Self::ContainerMetal,
        Self::ReceptacleMetal,
        Self::SackMultiWall,
        Self::Mat,
        Self::ReceptaclePlasticWrapped,
        Self::Matchbox,
        Self::NotAvailable,
        Self::UnpackedOrUnpackaged,
        Self::UnpackedOrUnpackagedSingleUnit,
        Self::UnpackedOrUnpackagedMultipleUnits,
        Self::Nest,
        Self::Net,
        Self::NetTubePlastic,
        Self::NetTubeTextile,
        Self::TwoSidedCageOnWheelsWithFixingStrap,
        Self::Trolley,
        Self::OnewayPalletISO012EUROPallet,
        Self::OnewayPalletISO111EUROPallet,
        Self::OnewayPalletISO221EUROPallet,
        Self::PalletWithExceptionalDimensions,
        Self::WoodenPallet40CmX80Cm,
        Self::PlasticPalletSRS60CmX80Cm,
        Self::PlasticPalletSRS80CmX120Cm,
        Self::PalletCHEP40CmX60Cm,
        Self::PalletCHEP80CmX120Cm,
        Self::PalletCHEP100CmX120Cm,
        Self::PalletAS40681993,
        Self::PalletISOT11,
        Self::PlatformUnspecifiedWeightOrDimension,
        Self::PalletISO012EUROPallet,
        Self::PalletISO111EUROPallet,
        Self::PalletISO221EUROPallet,
        Self::P14EUROPallet,
        Self::Block,
        Self::P18EUROPallet,
        Self::SyntheticPalletISO1,
        Self::SyntheticPalletISO2,
        Self::WholesalerPallet,
        Self::Pallet80X100Cm,
        Self::Pallet60X100Cm,
        Self::OnewayPallet,
        Self::Octabin,
        Self::ContainerOuter,
        Self::ReturnablePallet,
        Self::LargeBagPalletSized,
        Self::AWheeledPalletWithRaisedRim81X67X135,
        Self::AWheeledPalletWithRaisedRim81X72X135,
        Self::WheeledPalletWithRaisedRim81X60X16,
        Self::CHEPPallet60CmX80Cm,
        Self::Pan,
        Self::LPRPallet60CmX80Cm,
        Self::LPRPallet80CmX120Cm,
        Self::Packet,
        Self::PalletBoxCombinedOpenEndedBoxAndPallet,
        Self::Parcel,
        Self::PalletModularCollars80cms100cms,
        Self::PalletModularCollars80cms120cms,
        Self::Pen,
        Self::Plate,
        Self::Pitcher,
        Self::Pipe,
        Self::Punnet,
        Self::Package,
        Self::Pail,
        Self::Plank,
        Self::Pouch,
        Self::Piece,
        Self::ReceptaclePlastic,
        Self::Pot,
        Self::Tray,
        Self::PipesInBundleBunchTruss,
        Self::Pallet,
        Self::PlatesInBundleBunchTruss,
        Self::PlanksInBundleBunchTruss,
        Self::DrumSteelNonRemovableHead,
        Self::DrumSteelRemovableHead,
        Self::DrumAluminiumNonRemovableHead,
        Self::DrumAluminiumRemovableHead,
        Self::DrumPlasticNonRemovableHead,
        Self::DrumPlasticRemovableHead,
        Self::BarrelWoodenBungType,
        Self::BarrelWoodenRemovableHead,
        Self::JerricanSteelNonRemovableHead,
        Self::JerricanSteelRemovableHead,
        Self::JerricanPlasticNonRemovableHead,
        Self::JerricanPlasticRemovableHead,
        Self::BoxWoodenNaturalWoodOrdinary,
        Self::BoxWoodenNaturalWoodWithSiftProofWalls,
        Self::BoxPlasticExpanded,
        Self::BoxPlasticSolid,
        Self::Rod,
        Self::Ring,
        Self::RackClothingHanger,
        Self::Rack,
        Self::Reel,
        Self::Roll,
        Self::Rednet,
        Self::RodsInBundleBunchTruss,
        Self::Sack,
        Self::Slab,
        Self::CrateShallow,
        Self::Spindle,
        Self::SeaChest,
        Self::Sachet,
        Self::Skid,
        Self::CaseSkeleton,
        Self::Slipsheet,
        Self::Sheetmetal,
        Self::Spool,
        Self::SheetPlasticWrapping,
        Self::CaseSteel,
        Self::Sheet,
        Self::Suitcase,
        Self::EnvelopeSteel,
        Self::Shrinkwrapped,
        Self::Set,
        Self::Sleeve,
        Self::SheetsInBundleBunchTruss,
        Self::Tablet,
        Self::Tub,
        Self::TeaChest,
        Self::TubeCollapsible,
        Self::Tyre,
        Self::TankContainerGeneric,
        Self::Tierce,
        Self::TankRectangular,
        Self::TubWithLid,
        Self::Tin,
        Self::Tun,
        Self::Trunk,
        Self::Truss,
        Self::BagTote,
        Self::Tube,
        Self::TubeWithNozzle,
        Self::PalletTriwall,
        Self::TankCylindrical,
        Self::TubesInBundleBunchTruss,
        Self::Uncaged,
        Self::Unit,
        Self::Vat,
        Self::BulkGasAt1031MbarAnd15C,
        Self::Vial,
        Self::Vanpack,
        Self::BulkLiquid,
        Self::Vehicle,
        Self::BulkSolidLargeParticlesNodules,
        Self::VacuumPacked,
        Self::BulkLiquefiedGasAtAbnormalTemperaturePressure,
        Self::BulkSolidGranularParticlesGrains,
        Self::BulkScrapMetal,
        Self::BulkSolidFineParticlesPowders,
        Self::IntermediateBulkContainer,
        Self::Wickerbottle,
        Self::IntermediateBulkContainerSteel,
        Self::IntermediateBulkContainerAluminium,
        Self::IntermediateBulkContainerMetal,
        Self::IntermediateBulkContainerSteelPressurised10Kpa,
        Self::IntermediateBulkContainerAluminiumPressurised10Kpa,
        Self::IntermediateBulkContainerMetalPressure10Kpa,
        Self::IntermediateBulkContainerSteelLiquid,
        Self::IntermediateBulkContainerAluminiumLiquid,
        Self::IntermediateBulkContainerMetalLiquid,
        Self::IntermediateBulkContainerWovenPlasticWithoutCoatLiner,
        Self::IntermediateBulkContainerWovenPlasticCoated,
        Self::IntermediateBulkContainerWovenPlasticWithLiner,
        Self::IntermediateBulkContainerWovenPlasticCoatedAndLiner,
        Self::IntermediateBulkContainerPlasticFilm,
        Self::IntermediateBulkContainerTextileWithOutCoatLiner,
        Self::IntermediateBulkContainerNaturalWoodWithInnerLiner,
        Self::IntermediateBulkContainerTextileCoated,
        Self::IntermediateBulkContainerTextileWithLiner,
        Self::IntermediateBulkContainerTextileCoatedAndLiner,
        Self::IntermediateBulkContainerPlywoodWithInnerLiner,
        Self::IntermediateBulkContainerReconstitutedWoodWithInnerLiner,
        Self::BagWovenPlasticWithoutInnerCoatLiner,
        Self::BagWovenPlasticSiftProof,
        Self::BagWovenPlasticWaterResistant,
        Self::BagPlasticsFilm,
        Self::BagTextileWithoutInnerCoatLiner,
        Self::BagTextileSiftProof,
        Self::BagTextileWaterResistant,
        Self::BagPaperMultiWall,
        Self::BagPaperMultiWallWaterResistant,
        Self::CompositePackagingPlasticReceptacleInSteelDrum,
        Self::CompositePackagingPlasticReceptacleInSteelCrateBox,
        Self::CompositePackagingPlasticReceptacleInAluminiumDrum,
        Self::CompositePackagingPlasticReceptacleInAluminiumCrate,
        Self::CompositePackagingPlasticReceptacleInWoodenBox,
        Self::CompositePackagingPlasticReceptacleInPlywoodDrum,
        Self::CompositePackagingPlasticReceptacleInPlywoodBox,
        Self::CompositePackagingPlasticReceptacleInFibreDrum,
        Self::CompositePackagingPlasticReceptacleInFibreboardBox,
        Self::CompositePackagingPlasticReceptacleInPlasticDrum,
        Self::CompositePackagingPlasticReceptacleInSolidPlasticBox,
        Self::CompositePackagingGlassReceptacleInSteelDrum,
        Self::CompositePackagingGlassReceptacleInSteelCrateBox,
        Self::CompositePackagingGlassReceptacleInAluminiumDrum,
        Self::CompositePackagingGlassReceptacleInAluminiumCrate,
        Self::CompositePackagingGlassReceptacleInWoodenBox,
        Self::CompositePackagingGlassReceptacleInPlywoodDrum,
        Self::CompositePackagingGlassReceptacleInWickerworkHamper,
        Self::CompositePackagingGlassReceptacleInFibreDrum,
        Self::CompositePackagingGlassReceptacleInFibreboardBox,
        Self::CompositePackagingGlassReceptacleInExpandablePlasticPack,
        Self::CompositePackagingGlassReceptacleInSolidPlasticPack,
        Self::IntermediateBulkContainerPaperMultiWall,
        Self::BagLarge,
        Self::IntermediateBulkContainerPaperMultiWallWaterResistant,
        Self::IntermediateBulkContainerRigidPlasticWithStructuralEquipmentSolids,
        Self::IntermediateBulkContainerRigidPlasticFreestandingSolids,
        Self::IntermediateBulkContainerRigidPlasticWithStructuralEquipmentPressurised,
        Self::IntermediateBulkContainerRigidPlasticFreestandingPressurised,
        Self::IntermediateBulkContainerRigidPlasticWithStructuralEquipmentLiquids,
        Self::IntermediateBulkContainerRigidPlasticFreestandingLiquids,
        Self::IntermediateBulkContainerCompositeRigidPlasticSolids,
        Self::IntermediateBulkContainerCompositeFlexiblePlasticSolids,
        Self::IntermediateBulkContainerCompositeRigidPlasticPressurised,
        Self::IntermediateBulkContainerCompositeFlexiblePlasticPressurised,
        Self::IntermediateBulkContainerCompositeRigidPlasticLiquids,
        Self::IntermediateBulkContainerCompositeFlexiblePlasticLiquids,
        Self::IntermediateBulkContainerComposite,
        Self::IntermediateBulkContainerFibreboard,
        Self::IntermediateBulkContainerFlexible,
        Self::IntermediateBulkContainerMetalOtherThanSteel,
        Self::IntermediateBulkContainerNaturalWood,
        Self::IntermediateBulkContainerPlywood,
        Self::IntermediateBulkContainerReconstitutedWood,
        Self::MutuallyDefined,
    ];

    /// Resolves a package type by its `X`-prefixed code, or `None` if unknown.
    pub fn from_code(code: &str) -> Option<Self> {
        Some(match code {
            "X1" => Self::X1,
            "X1A" => Self::DrumSteel,
            "X1B" => Self::DrumAluminium,
            "X1D" => Self::DrumPlywood,
            "X1F" => Self::ContainerFlexible,
            "X1G" => Self::DrumFibre,
            "X1W" => Self::DrumWooden,
            "X2C" => Self::BarrelWooden,
            "X3A" => Self::JerricanSteel,
            "X3H" => Self::JerricanPlastic,
            "X43" => Self::BagSuperBulk,
            "X44" => Self::BagPolybag,
            "X4A" => Self::BoxSteel,
            "X4B" => Self::BoxAluminium,
            "X4C" => Self::BoxNaturalWood,
            "X4D" => Self::BoxPlywood,
            "X4F" => Self::BoxReconstitutedWood,
            "X4G" => Self::BoxFibreboard,
            "X4H" => Self::BoxPlastic,
            "X5H" => Self::BagWovenPlastic,
            "X5L" => Self::BagTextile,
            "X5M" => Self::BagPaper,
            "X6H" => Self::CompositePackagingPlasticReceptacle,
            "X6P" => Self::CompositePackagingGlassReceptacle,
            "X7A" => Self::CaseCar,
            "X7B" => Self::CaseWooden,
            "X8A" => Self::PalletWooden,
            "X8B" => Self::CrateWooden,
            "X8C" => Self::BundleWooden,
            "XAA" => Self::IntermediateBulkContainerRigidPlastic,
            "XAB" => Self::ReceptacleFibre,
            "XAC" => Self::ReceptaclePaper,
            "XAD" => Self::ReceptacleWooden,
            "XAE" => Self::Aerosol,
            "XAF" => Self::PalletModularCollars80cms60cms,
            "XAG" => Self::PalletShrinkwrapped,
            "XAH" => Self::Pallet100cms110cms,
            "XAI" => Self::Clamshell,
            "XAJ" => Self::Cone,
            "XAL" => Self::Ball,
            "XAM" => Self::AmpouleNonProtected,
            "XAP" => Self::AmpouleProtected,
            "XAT" => Self::Atomizer,
            "XAV" => Self::Capsule,
            "XB4" => Self::Belt,
            "XBA" => Self::Barrel,
            "XBB" => Self::Bobbin,
            "XBC" => Self::BottlecrateBottlerack,
            "XBD" => Self::Board,
            "XBE" => Self::Bundle,
            "XBF" => Self::BalloonNonProtected,
            "XBG" => Self::Bag,
            "XBH" => Self::Bunch,
            "XBI" => Self::Bin,
            "XBJ" => Self::Bucket,
            "XBK" => Self::Basket,
            "XBL" => Self::BaleCompressed,
            "XBM" => Self::Basin,
            "XBN" => Self::BaleNonCompressed,
            "XBO" => Self::BottleNonProtectedCylindrical,
            "XBP" => Self::BalloonProtected,
            "XBQ" => Self::BottleProtectedCylindrical,
            "XBR" => Self::Bar,
            "XBS" => Self::BottleNonProtectedBulbous,
            "XBT" => Self::Bolt,
            "XBU" => Self::Butt,
            "XBV" => Self::BottleProtectedBulbous,
            "XBW" => Self::BoxForLiquids,
            "XBX" => Self::Box,
            "XBY" => Self::BoardInBundleBunchTruss,
            "XBZ" => Self::BarsInBundleBunchTruss,
            "XCA" => Self::CanRectangular,
            "XCB" => Self::CrateBeer,
            "XCC" => Self::Churn,
            "XCD" => Self::CanWithHandleAndSpout,
            "XCE" => Self::Creel,
            "XCF" => Self::Coffer,
            "XCG" => Self::Cage,
            "XCH" => Self::Chest,
            "XCI" => Self::Canister,
            "XCJ" => Self::Coffin,
            "XCK" => Self::Cask,
            "XCL" => Self::Coil,
            "XCM" => Self::Card,
            "XCN" => Self::ContainerNotOtherwiseSpecifiedAsTransportEquipment,
            "XCO" => Self::CarboyNonProtected,
            "XCP" => Self::CarboyProtected,
            "XCQ" => Self::Cartridge,
            "XCR" => Self::Crate,
            "XCS" => Self::Case,
            "XCT" => Self::Carton,
            "XCU" => Self::Cup,
            "XCV" => Self::Cover,
            "XCW" => Self::CageRoll,
            "XCX" => Self::CanCylindrical,
            "XCY" => Self::Cylinder,
            "XCZ" => Self::Canvas,
            "XDA" => Self::CrateMultipleLayerPlastic,
            "XDB" => Self::CrateMultipleLayerWooden,
            "XDC" => Self::CrateMultipleLayerCardboard,
            "XDG" => Self::CageCommonwealthHandlingEquipmentPoolCHEP,
            "XDH" => Self::BoxCommonwealthHandlingEquipmentPoolCHEPEurobox,
            "XDI" => Self::DrumIron,
            "XDJ" => Self::DemijohnNonProtected,
            "XDK" => Self::CrateBulkCardboard,
            "XDL" => Self::CrateBulkPlastic,
            "XDM" => Self::CrateBulkWooden,
            "XDN" => Self::Dispenser,
            "XDP" => Self::DemijohnProtected,
            "XDR" => Self::Drum,
            "XDS" => Self::TrayOneLayerNoCoverPlastic,
            "XDT" => Self::TrayOneLayerNoCoverWooden,
            "XDU" => Self::TrayOneLayerNoCoverPolystyrene,
            "XDV" => Self::TrayOneLayerNoCoverCardboard,
            "XDW" => Self::TrayTwoLayersNoCoverPlasticTray,
            "XDX" => Self::TrayTwoLayersNoCoverWooden,
            "XDY" => Self::TrayTwoLayersNoCoverCardboard,
            "XEC" => Self::BagPlastic,
            "XED" => Self::CaseWithPalletBase,
            "XEE" => Self::CaseWithPalletBaseWooden,
            "XEF" => Self::CaseWithPalletBaseCardboard,
            "XEG" => Self::CaseWithPalletBasePlastic,
            "XEH" => Self::CaseWithPalletBaseMetal,
            "XEI" => Self::CaseIsothermic,
            "XEN" => Self::Envelope,
            "XFB" => Self::Flexibag,
            "XFC" => Self::CrateFruit,
            "XFD" => Self::CrateFramed,
            "XFE" => Self::Flexitank,
            "XFI" => Self::Firkin,
            "XFL" => Self::Flask,
            "XFO" => Self::Footlocker,
            "XFP" => Self::Filmpack,
            "XFR" => Self::Frame,
            "XFT" => Self::Foodtainer,
            "XFW" => Self::CartFlatbed,
            "XFX" => Self::BagFlexibleContainer,
            "XGB" => Self::BottleGas,
            "XGI" => Self::Girder,
            "XGL" => Self::ContainerGallon,
            "XGR" => Self::ReceptacleGlass,
            "XGU" => Self::TrayContainingHorizontallyStackedFlatItems,
            "XGY" => Self::BagGunny,
            "XGZ" => Self::GirdersInBundleBunchTruss,
            "XHA" => Self::BasketWithHandlePlastic,
            "XHB" => Self::BasketWithHandleWooden,
            "XHC" => Self::BasketWithHandleCardboard,
            "XHG" => Self::Hogshead,
            "XHN" => Self::Hanger,
            "XHR" => Self::Hamper,
            "XIA" => Self::PackageDisplayWooden,
            "XIB" => Self::PackageDisplayCardboard,
            "XIC" => Self::PackageDisplayPlastic,
            "XID" => Self::PackageDisplayMetal,
            "XIE" => Self::PackageShow,
            "XIF" => Self::PackageFlow,
            "XIG" => Self::PackagePaperWrapped,
            "XIH" => Self::DrumPlastic,
            "XIK" => Self::PackageCardboardWithBottleGripHoles,
            "XIL" => Self::TrayRigidLiddedStackableCENTS144822002,
            "XIN" => Self::Ingot,
            "XIZ" => Self::IngotsInBundleBunchTruss,
            "XJB" => Self::BagJumbo,
            "XJC" => Self::JerricanRectangular,
            "XJG" => Self::Jug,
            "XJR" => Self::Jar,
            "XJT" => Self::Jutebag,
            "XJY" => Self::JerricanCylindrical,
            "XKG" => Self::Keg,
            "XKI" => Self::Kit,
            "XLE" => Self::Luggage,
            "XLG" => Self::Log,
            "XLT" => Self::Lot,
            "XLU" => Self::Lug,
            "XLV" => Self::Liftvan,
            "XLZ" => Self::LogsInBundleBunchTruss,
            "XMA" => Self::CrateMetal,
            "XMB" => Self::BagMultiply,
            "XMC" => Self::CrateMilk,
            "XME" => Self::ContainerMetal,
            "XMR" => Self::ReceptacleMetal,
            "XMS" => Self::SackMultiWall,
            "XMT" => Self::Mat,
            "XMW" => Self::ReceptaclePlasticWrapped,
            "XMX" => Self::Matchbox,
            "XNA" => Self::NotAvailable,
            "XNE" => Self::UnpackedOrUnpackaged,
            "XNF" => Self::UnpackedOrUnpackagedSingleUnit,
            "XNG" => Self::UnpackedOrUnpackagedMultipleUnits,
            "XNS" => Self::Nest,
            "XNT" => Self::Net,
            "XNU" => Self::NetTubePlastic,
            "XNV" => Self::NetTubeTextile,
            "XO1" => Self::TwoSidedCageOnWheelsWithFixingStrap,
            "XO2" => Self::Trolley,
            "XO3" => Self::OnewayPalletISO012EUROPallet,
            "XO4" => Self::OnewayPalletISO111EUROPallet,
            "XO5" => Self::OnewayPalletISO221EUROPallet,
            "XO6" => Self::PalletWithExceptionalDimensions,
            "XO7" => Self::WoodenPallet40CmX80Cm,
            "XO8" => Self::PlasticPalletSRS60CmX80Cm,
            "XO9" => Self::PlasticPalletSRS80CmX120Cm,
            "XOA" => Self::PalletCHEP40CmX60Cm,
            "XOB" => Self::PalletCHEP80CmX120Cm,
            "XOC" => Self::PalletCHEP100CmX120Cm,
            "XOD" => Self::PalletAS40681993,
            "XOE" => Self::PalletISOT11,
            "XOF" => Self::PlatformUnspecifiedWeightOrDimension,
            "XOG" => Self::PalletISO012EUROPallet,
            "XOH" => Self::PalletISO111EUROPallet,
            "XOI" => Self::PalletISO221EUROPallet,
            "XOJ" => Self::P14EUROPallet,
            "XOK" => Self::Block,
            "XOL" => Self::P18EUROPallet,
            "XOM" => Self::SyntheticPalletISO1,
            "XON" => Self::SyntheticPalletISO2,
            "XOP" => Self::WholesalerPallet,
            "XOQ" => Self::Pallet80X100Cm,
            "XOR" => Self::Pallet60X100Cm,
            "XOS" => Self::OnewayPallet,
            "XOT" => Self::Octabin,
            "XOU" => Self::ContainerOuter,
            "XOV" => Self::ReturnablePallet,
            "XOW" => Self::LargeBagPalletSized,
            "XOX" => Self::AWheeledPalletWithRaisedRim81X67X135,
            "XOY" => Self::AWheeledPalletWithRaisedRim81X72X135,
            "XOZ" => Self::WheeledPalletWithRaisedRim81X60X16,
            "XP1" => Self::CHEPPallet60CmX80Cm,
            "XP2" => Self::Pan,
            "XP3" => Self::LPRPallet60CmX80Cm,
            "XP4" => Self::LPRPallet80CmX120Cm,
            "XPA" => Self::Packet,
            "XPB" => Self::PalletBoxCombinedOpenEndedBoxAndPallet,
            "XPC" => Self::Parcel,
            "XPD" => Self::PalletModularCollars80cms100cms,
            "XPE" => Self::PalletModularCollars80cms120cms,
            "XPF" => Self::Pen,
            "XPG" => Self::Plate,
            "XPH" => Self::Pitcher,
            "XPI" => Self::Pipe,
            "XPJ" => Self::Punnet,
            "XPK" => Self::Package,
            "XPL" => Self::Pail,
            "XPN" => Self::Plank,
            "XPO" => Self::Pouch,
            "XPP" => Self::Piece,
            "XPR" => Self::ReceptaclePlastic,
            "XPT" => Self::Pot,
            "XPU" => Self::Tray,
            "XPV" => Self::PipesInBundleBunchTruss,
            "XPX" => Self::Pallet,
            "XPY" => Self::PlatesInBundleBunchTruss,
            "XPZ" => Self::PlanksInBundleBunchTruss,
            "XQA" => Self::DrumSteelNonRemovableHead,
            "XQB" => Self::DrumSteelRemovableHead,
            "XQC" => Self::DrumAluminiumNonRemovableHead,
            "XQD" => Self::DrumAluminiumRemovableHead,
            "XQF" => Self::DrumPlasticNonRemovableHead,
            "XQG" => Self::DrumPlasticRemovableHead,
            "XQH" => Self::BarrelWoodenBungType,
            "XQJ" => Self::BarrelWoodenRemovableHead,
            "XQK" => Self::JerricanSteelNonRemovableHead,
            "XQL" => Self::JerricanSteelRemovableHead,
            "XQM" => Self::JerricanPlasticNonRemovableHead,
            "XQN" => Self::JerricanPlasticRemovableHead,
            "XQP" => Self::BoxWoodenNaturalWoodOrdinary,
            "XQQ" => Self::BoxWoodenNaturalWoodWithSiftProofWalls,
            "XQR" => Self::BoxPlasticExpanded,
            "XQS" => Self::BoxPlasticSolid,
            "XRD" => Self::Rod,
            "XRG" => Self::Ring,
            "XRJ" => Self::RackClothingHanger,
            "XRK" => Self::Rack,
            "XRL" => Self::Reel,
            "XRO" => Self::Roll,
            "XRT" => Self::Rednet,
            "XRZ" => Self::RodsInBundleBunchTruss,
            "XSA" => Self::Sack,
            "XSB" => Self::Slab,
            "XSC" => Self::CrateShallow,
            "XSD" => Self::Spindle,
            "XSE" => Self::SeaChest,
            "XSH" => Self::Sachet,
            "XSI" => Self::Skid,
            "XSK" => Self::CaseSkeleton,
            "XSL" => Self::Slipsheet,
            "XSM" => Self::Sheetmetal,
            "XSO" => Self::Spool,
            "XSP" => Self::SheetPlasticWrapping,
            "XSS" => Self::CaseSteel,
            "XST" => Self::Sheet,
            "XSU" => Self::Suitcase,
            "XSV" => Self::EnvelopeSteel,
            "XSW" => Self::Shrinkwrapped,
            "XSX" => Self::Set,
            "XSY" => Self::Sleeve,
            "XSZ" => Self::SheetsInBundleBunchTruss,
            "XT1" => Self::Tablet,
            "XTB" => Self::Tub,
            "XTC" => Self::TeaChest,
            "XTD" => Self::TubeCollapsible,
            "XTE" => Self::Tyre,
            "XTG" => Self::TankContainerGeneric,
            "XTI" => Self::Tierce,
            "XTK" => Self::TankRectangular,
            "XTL" => Self::TubWithLid,
            "XTN" => Self::Tin,
            "XTO" => Self::Tun,
            "XTR" => Self::Trunk,
            "XTS" => Self::Truss,
            "XTT" => Self::BagTote,
            "XTU" => Self::Tube,
            "XTV" => Self::TubeWithNozzle,
            "XTW" => Self::PalletTriwall,
            "XTY" => Self::TankCylindrical,
            "XTZ" => Self::TubesInBundleBunchTruss,
            "XUC" => Self::Uncaged,
            "XUN" => Self::Unit,
            "XVA" => Self::Vat,
            "XVG" => Self::BulkGasAt1031MbarAnd15C,
            "XVI" => Self::Vial,
            "XVK" => Self::Vanpack,
            "XVL" => Self::BulkLiquid,
            "XVN" => Self::Vehicle,
            "XVO" => Self::BulkSolidLargeParticlesNodules,
            "XVP" => Self::VacuumPacked,
            "XVQ" => Self::BulkLiquefiedGasAtAbnormalTemperaturePressure,
            "XVR" => Self::BulkSolidGranularParticlesGrains,
            "XVS" => Self::BulkScrapMetal,
            "XVY" => Self::BulkSolidFineParticlesPowders,
            "XWA" => Self::IntermediateBulkContainer,
            "XWB" => Self::Wickerbottle,
            "XWC" => Self::IntermediateBulkContainerSteel,
            "XWD" => Self::IntermediateBulkContainerAluminium,
            "XWF" => Self::IntermediateBulkContainerMetal,
            "XWG" => Self::IntermediateBulkContainerSteelPressurised10Kpa,
            "XWH" => Self::IntermediateBulkContainerAluminiumPressurised10Kpa,
            "XWJ" => Self::IntermediateBulkContainerMetalPressure10Kpa,
            "XWK" => Self::IntermediateBulkContainerSteelLiquid,
            "XWL" => Self::IntermediateBulkContainerAluminiumLiquid,
            "XWM" => Self::IntermediateBulkContainerMetalLiquid,
            "XWN" => Self::IntermediateBulkContainerWovenPlasticWithoutCoatLiner,
            "XWP" => Self::IntermediateBulkContainerWovenPlasticCoated,
            "XWQ" => Self::IntermediateBulkContainerWovenPlasticWithLiner,
            "XWR" => Self::IntermediateBulkContainerWovenPlasticCoatedAndLiner,
            "XWS" => Self::IntermediateBulkContainerPlasticFilm,
            "XWT" => Self::IntermediateBulkContainerTextileWithOutCoatLiner,
            "XWU" => Self::IntermediateBulkContainerNaturalWoodWithInnerLiner,
            "XWV" => Self::IntermediateBulkContainerTextileCoated,
            "XWW" => Self::IntermediateBulkContainerTextileWithLiner,
            "XWX" => Self::IntermediateBulkContainerTextileCoatedAndLiner,
            "XWY" => Self::IntermediateBulkContainerPlywoodWithInnerLiner,
            "XWZ" => Self::IntermediateBulkContainerReconstitutedWoodWithInnerLiner,
            "XXA" => Self::BagWovenPlasticWithoutInnerCoatLiner,
            "XXB" => Self::BagWovenPlasticSiftProof,
            "XXC" => Self::BagWovenPlasticWaterResistant,
            "XXD" => Self::BagPlasticsFilm,
            "XXF" => Self::BagTextileWithoutInnerCoatLiner,
            "XXG" => Self::BagTextileSiftProof,
            "XXH" => Self::BagTextileWaterResistant,
            "XXJ" => Self::BagPaperMultiWall,
            "XXK" => Self::BagPaperMultiWallWaterResistant,
            "XYA" => Self::CompositePackagingPlasticReceptacleInSteelDrum,
            "XYB" => Self::CompositePackagingPlasticReceptacleInSteelCrateBox,
            "XYC" => Self::CompositePackagingPlasticReceptacleInAluminiumDrum,
            "XYD" => Self::CompositePackagingPlasticReceptacleInAluminiumCrate,
            "XYF" => Self::CompositePackagingPlasticReceptacleInWoodenBox,
            "XYG" => Self::CompositePackagingPlasticReceptacleInPlywoodDrum,
            "XYH" => Self::CompositePackagingPlasticReceptacleInPlywoodBox,
            "XYJ" => Self::CompositePackagingPlasticReceptacleInFibreDrum,
            "XYK" => Self::CompositePackagingPlasticReceptacleInFibreboardBox,
            "XYL" => Self::CompositePackagingPlasticReceptacleInPlasticDrum,
            "XYM" => Self::CompositePackagingPlasticReceptacleInSolidPlasticBox,
            "XYN" => Self::CompositePackagingGlassReceptacleInSteelDrum,
            "XYP" => Self::CompositePackagingGlassReceptacleInSteelCrateBox,
            "XYQ" => Self::CompositePackagingGlassReceptacleInAluminiumDrum,
            "XYR" => Self::CompositePackagingGlassReceptacleInAluminiumCrate,
            "XYS" => Self::CompositePackagingGlassReceptacleInWoodenBox,
            "XYT" => Self::CompositePackagingGlassReceptacleInPlywoodDrum,
            "XYV" => Self::CompositePackagingGlassReceptacleInWickerworkHamper,
            "XYW" => Self::CompositePackagingGlassReceptacleInFibreDrum,
            "XYX" => Self::CompositePackagingGlassReceptacleInFibreboardBox,
            "XYY" => Self::CompositePackagingGlassReceptacleInExpandablePlasticPack,
            "XYZ" => Self::CompositePackagingGlassReceptacleInSolidPlasticPack,
            "XZA" => Self::IntermediateBulkContainerPaperMultiWall,
            "XZB" => Self::BagLarge,
            "XZC" => Self::IntermediateBulkContainerPaperMultiWallWaterResistant,
            "XZD" => Self::IntermediateBulkContainerRigidPlasticWithStructuralEquipmentSolids,
            "XZF" => Self::IntermediateBulkContainerRigidPlasticFreestandingSolids,
            "XZG" => Self::IntermediateBulkContainerRigidPlasticWithStructuralEquipmentPressurised,
            "XZH" => Self::IntermediateBulkContainerRigidPlasticFreestandingPressurised,
            "XZJ" => Self::IntermediateBulkContainerRigidPlasticWithStructuralEquipmentLiquids,
            "XZK" => Self::IntermediateBulkContainerRigidPlasticFreestandingLiquids,
            "XZL" => Self::IntermediateBulkContainerCompositeRigidPlasticSolids,
            "XZM" => Self::IntermediateBulkContainerCompositeFlexiblePlasticSolids,
            "XZN" => Self::IntermediateBulkContainerCompositeRigidPlasticPressurised,
            "XZP" => Self::IntermediateBulkContainerCompositeFlexiblePlasticPressurised,
            "XZQ" => Self::IntermediateBulkContainerCompositeRigidPlasticLiquids,
            "XZR" => Self::IntermediateBulkContainerCompositeFlexiblePlasticLiquids,
            "XZS" => Self::IntermediateBulkContainerComposite,
            "XZT" => Self::IntermediateBulkContainerFibreboard,
            "XZU" => Self::IntermediateBulkContainerFlexible,
            "XZV" => Self::IntermediateBulkContainerMetalOtherThanSteel,
            "XZW" => Self::IntermediateBulkContainerNaturalWood,
            "XZX" => Self::IntermediateBulkContainerPlywood,
            "XZY" => Self::IntermediateBulkContainerReconstitutedWood,
            "XZZ" => Self::MutuallyDefined,
            _ => return None,
        })
    }

    /// The `X`-prefixed code of this package type.
    pub fn code(self) -> &'static str {
        match self {
            Self::X1 => "X1",
            Self::DrumSteel => "X1A",
            Self::DrumAluminium => "X1B",
            Self::DrumPlywood => "X1D",
            Self::ContainerFlexible => "X1F",
            Self::DrumFibre => "X1G",
            Self::DrumWooden => "X1W",
            Self::BarrelWooden => "X2C",
            Self::JerricanSteel => "X3A",
            Self::JerricanPlastic => "X3H",
            Self::BagSuperBulk => "X43",
            Self::BagPolybag => "X44",
            Self::BoxSteel => "X4A",
            Self::BoxAluminium => "X4B",
            Self::BoxNaturalWood => "X4C",
            Self::BoxPlywood => "X4D",
            Self::BoxReconstitutedWood => "X4F",
            Self::BoxFibreboard => "X4G",
            Self::BoxPlastic => "X4H",
            Self::BagWovenPlastic => "X5H",
            Self::BagTextile => "X5L",
            Self::BagPaper => "X5M",
            Self::CompositePackagingPlasticReceptacle => "X6H",
            Self::CompositePackagingGlassReceptacle => "X6P",
            Self::CaseCar => "X7A",
            Self::CaseWooden => "X7B",
            Self::PalletWooden => "X8A",
            Self::CrateWooden => "X8B",
            Self::BundleWooden => "X8C",
            Self::IntermediateBulkContainerRigidPlastic => "XAA",
            Self::ReceptacleFibre => "XAB",
            Self::ReceptaclePaper => "XAC",
            Self::ReceptacleWooden => "XAD",
            Self::Aerosol => "XAE",
            Self::PalletModularCollars80cms60cms => "XAF",
            Self::PalletShrinkwrapped => "XAG",
            Self::Pallet100cms110cms => "XAH",
            Self::Clamshell => "XAI",
            Self::Cone => "XAJ",
            Self::Ball => "XAL",
            Self::AmpouleNonProtected => "XAM",
            Self::AmpouleProtected => "XAP",
            Self::Atomizer => "XAT",
            Self::Capsule => "XAV",
            Self::Belt => "XB4",
            Self::Barrel => "XBA",
            Self::Bobbin => "XBB",
            Self::BottlecrateBottlerack => "XBC",
            Self::Board => "XBD",
            Self::Bundle => "XBE",
            Self::BalloonNonProtected => "XBF",
            Self::Bag => "XBG",
            Self::Bunch => "XBH",
            Self::Bin => "XBI",
            Self::Bucket => "XBJ",
            Self::Basket => "XBK",
            Self::BaleCompressed => "XBL",
            Self::Basin => "XBM",
            Self::BaleNonCompressed => "XBN",
            Self::BottleNonProtectedCylindrical => "XBO",
            Self::BalloonProtected => "XBP",
            Self::BottleProtectedCylindrical => "XBQ",
            Self::Bar => "XBR",
            Self::BottleNonProtectedBulbous => "XBS",
            Self::Bolt => "XBT",
            Self::Butt => "XBU",
            Self::BottleProtectedBulbous => "XBV",
            Self::BoxForLiquids => "XBW",
            Self::Box => "XBX",
            Self::BoardInBundleBunchTruss => "XBY",
            Self::BarsInBundleBunchTruss => "XBZ",
            Self::CanRectangular => "XCA",
            Self::CrateBeer => "XCB",
            Self::Churn => "XCC",
            Self::CanWithHandleAndSpout => "XCD",
            Self::Creel => "XCE",
            Self::Coffer => "XCF",
            Self::Cage => "XCG",
            Self::Chest => "XCH",
            Self::Canister => "XCI",
            Self::Coffin => "XCJ",
            Self::Cask => "XCK",
            Self::Coil => "XCL",
            Self::Card => "XCM",
            Self::ContainerNotOtherwiseSpecifiedAsTransportEquipment => "XCN",
            Self::CarboyNonProtected => "XCO",
            Self::CarboyProtected => "XCP",
            Self::Cartridge => "XCQ",
            Self::Crate => "XCR",
            Self::Case => "XCS",
            Self::Carton => "XCT",
            Self::Cup => "XCU",
            Self::Cover => "XCV",
            Self::CageRoll => "XCW",
            Self::CanCylindrical => "XCX",
            Self::Cylinder => "XCY",
            Self::Canvas => "XCZ",
            Self::CrateMultipleLayerPlastic => "XDA",
            Self::CrateMultipleLayerWooden => "XDB",
            Self::CrateMultipleLayerCardboard => "XDC",
            Self::CageCommonwealthHandlingEquipmentPoolCHEP => "XDG",
            Self::BoxCommonwealthHandlingEquipmentPoolCHEPEurobox => "XDH",
            Self::DrumIron => "XDI",
            Self::DemijohnNonProtected => "XDJ",
            Self::CrateBulkCardboard => "XDK",
            Self::CrateBulkPlastic => "XDL",
            Self::CrateBulkWooden => "XDM",
            Self::Dispenser => "XDN",
            Self::DemijohnProtected => "XDP",
            Self::Drum => "XDR",
            Self::TrayOneLayerNoCoverPlastic => "XDS",
            Self::TrayOneLayerNoCoverWooden => "XDT",
            Self::TrayOneLayerNoCoverPolystyrene => "XDU",
            Self::TrayOneLayerNoCoverCardboard => "XDV",
            Self::TrayTwoLayersNoCoverPlasticTray => "XDW",
            Self::TrayTwoLayersNoCoverWooden => "XDX",
            Self::TrayTwoLayersNoCoverCardboard => "XDY",
            Self::BagPlastic => "XEC",
            Self::CaseWithPalletBase => "XED",
            Self::CaseWithPalletBaseWooden => "XEE",
            Self::CaseWithPalletBaseCardboard => "XEF",
            Self::CaseWithPalletBasePlastic => "XEG",
            Self::CaseWithPalletBaseMetal => "XEH",
            Self::CaseIsothermic => "XEI",
            Self::Envelope => "XEN",
            Self::Flexibag => "XFB",
            Self::CrateFruit => "XFC",
            Self::CrateFramed => "XFD",
            Self::Flexitank => "XFE",
            Self::Firkin => "XFI",
            Self::Flask => "XFL",
            Self::Footlocker => "XFO",
            Self::Filmpack => "XFP",
            Self::Frame => "XFR",
            Self::Foodtainer => "XFT",
            Self::CartFlatbed => "XFW",
            Self::BagFlexibleContainer => "XFX",
            Self::BottleGas => "XGB",
            Self::Girder => "XGI",
            Self::ContainerGallon => "XGL",
            Self::ReceptacleGlass => "XGR",
            Self::TrayContainingHorizontallyStackedFlatItems => "XGU",
            Self::BagGunny => "XGY",
            Self::GirdersInBundleBunchTruss => "XGZ",
            Self::BasketWithHandlePlastic => "XHA",
            Self::BasketWithHandleWooden => "XHB",
            Self::BasketWithHandleCardboard => "XHC",
            Self::Hogshead => "XHG",
            Self::Hanger => "XHN",
            Self::Hamper => "XHR",
            Self::PackageDisplayWooden => "XIA",
            Self::PackageDisplayCardboard => "XIB",
            Self::PackageDisplayPlastic => "XIC",
            Self::PackageDisplayMetal => "XID",
            Self::PackageShow => "XIE",
            Self::PackageFlow => "XIF",
            Self::PackagePaperWrapped => "XIG",
            Self::DrumPlastic => "XIH",
            Self::PackageCardboardWithBottleGripHoles => "XIK",
            Self::TrayRigidLiddedStackableCENTS144822002 => "XIL",
            Self::Ingot => "XIN",
            Self::IngotsInBundleBunchTruss => "XIZ",
            Self::BagJumbo => "XJB",
            Self::JerricanRectangular => "XJC",
            Self::Jug => "XJG",
            Self::Jar => "XJR",
            Self::Jutebag => "XJT",
            Self::JerricanCylindrical => "XJY",
            Self::Keg => "XKG",
            Self::Kit => "XKI",
            Self::Luggage => "XLE",
            Self::Log => "XLG",
            Self::Lot => "XLT",
            Self::Lug => "XLU",
            Self::Liftvan => "XLV",
            Self::LogsInBundleBunchTruss => "XLZ",
            Self::CrateMetal => "XMA",
            Self::BagMultiply => "XMB",
            Self::CrateMilk => "XMC",
            Self::ContainerMetal => "XME",
            Self::ReceptacleMetal => "XMR",
            Self::SackMultiWall => "XMS",
            Self::Mat => "XMT",
            Self::ReceptaclePlasticWrapped => "XMW",
            Self::Matchbox => "XMX",
            Self::NotAvailable => "XNA",
            Self::UnpackedOrUnpackaged => "XNE",
            Self::UnpackedOrUnpackagedSingleUnit => "XNF",
            Self::UnpackedOrUnpackagedMultipleUnits => "XNG",
            Self::Nest => "XNS",
            Self::Net => "XNT",
            Self::NetTubePlastic => "XNU",
            Self::NetTubeTextile => "XNV",
            Self::TwoSidedCageOnWheelsWithFixingStrap => "XO1",
            Self::Trolley => "XO2",
            Self::OnewayPalletISO012EUROPallet => "XO3",
            Self::OnewayPalletISO111EUROPallet => "XO4",
            Self::OnewayPalletISO221EUROPallet => "XO5",
            Self::PalletWithExceptionalDimensions => "XO6",
            Self::WoodenPallet40CmX80Cm => "XO7",
            Self::PlasticPalletSRS60CmX80Cm => "XO8",
            Self::PlasticPalletSRS80CmX120Cm => "XO9",
            Self::PalletCHEP40CmX60Cm => "XOA",
            Self::PalletCHEP80CmX120Cm => "XOB",
            Self::PalletCHEP100CmX120Cm => "XOC",
            Self::PalletAS40681993 => "XOD",
            Self::PalletISOT11 => "XOE",
            Self::PlatformUnspecifiedWeightOrDimension => "XOF",
            Self::PalletISO012EUROPallet => "XOG",
            Self::PalletISO111EUROPallet => "XOH",
            Self::PalletISO221EUROPallet => "XOI",
            Self::P14EUROPallet => "XOJ",
            Self::Block => "XOK",
            Self::P18EUROPallet => "XOL",
            Self::SyntheticPalletISO1 => "XOM",
            Self::SyntheticPalletISO2 => "XON",
            Self::WholesalerPallet => "XOP",
            Self::Pallet80X100Cm => "XOQ",
            Self::Pallet60X100Cm => "XOR",
            Self::OnewayPallet => "XOS",
            Self::Octabin => "XOT",
            Self::ContainerOuter => "XOU",
            Self::ReturnablePallet => "XOV",
            Self::LargeBagPalletSized => "XOW",
            Self::AWheeledPalletWithRaisedRim81X67X135 => "XOX",
            Self::AWheeledPalletWithRaisedRim81X72X135 => "XOY",
            Self::WheeledPalletWithRaisedRim81X60X16 => "XOZ",
            Self::CHEPPallet60CmX80Cm => "XP1",
            Self::Pan => "XP2",
            Self::LPRPallet60CmX80Cm => "XP3",
            Self::LPRPallet80CmX120Cm => "XP4",
            Self::Packet => "XPA",
            Self::PalletBoxCombinedOpenEndedBoxAndPallet => "XPB",
            Self::Parcel => "XPC",
            Self::PalletModularCollars80cms100cms => "XPD",
            Self::PalletModularCollars80cms120cms => "XPE",
            Self::Pen => "XPF",
            Self::Plate => "XPG",
            Self::Pitcher => "XPH",
            Self::Pipe => "XPI",
            Self::Punnet => "XPJ",
            Self::Package => "XPK",
            Self::Pail => "XPL",
            Self::Plank => "XPN",
            Self::Pouch => "XPO",
            Self::Piece => "XPP",
            Self::ReceptaclePlastic => "XPR",
            Self::Pot => "XPT",
            Self::Tray => "XPU",
            Self::PipesInBundleBunchTruss => "XPV",
            Self::Pallet => "XPX",
            Self::PlatesInBundleBunchTruss => "XPY",
            Self::PlanksInBundleBunchTruss => "XPZ",
            Self::DrumSteelNonRemovableHead => "XQA",
            Self::DrumSteelRemovableHead => "XQB",
            Self::DrumAluminiumNonRemovableHead => "XQC",
            Self::DrumAluminiumRemovableHead => "XQD",
            Self::DrumPlasticNonRemovableHead => "XQF",
            Self::DrumPlasticRemovableHead => "XQG",
            Self::BarrelWoodenBungType => "XQH",
            Self::BarrelWoodenRemovableHead => "XQJ",
            Self::JerricanSteelNonRemovableHead => "XQK",
            Self::JerricanSteelRemovableHead => "XQL",
            Self::JerricanPlasticNonRemovableHead => "XQM",
            Self::JerricanPlasticRemovableHead => "XQN",
            Self::BoxWoodenNaturalWoodOrdinary => "XQP",
            Self::BoxWoodenNaturalWoodWithSiftProofWalls => "XQQ",
            Self::BoxPlasticExpanded => "XQR",
            Self::BoxPlasticSolid => "XQS",
            Self::Rod => "XRD",
            Self::Ring => "XRG",
            Self::RackClothingHanger => "XRJ",
            Self::Rack => "XRK",
            Self::Reel => "XRL",
            Self::Roll => "XRO",
            Self::Rednet => "XRT",
            Self::RodsInBundleBunchTruss => "XRZ",
            Self::Sack => "XSA",
            Self::Slab => "XSB",
            Self::CrateShallow => "XSC",
            Self::Spindle => "XSD",
            Self::SeaChest => "XSE",
            Self::Sachet => "XSH",
            Self::Skid => "XSI",
            Self::CaseSkeleton => "XSK",
            Self::Slipsheet => "XSL",
            Self::Sheetmetal => "XSM",
            Self::Spool => "XSO",
            Self::SheetPlasticWrapping => "XSP",
            Self::CaseSteel => "XSS",
            Self::Sheet => "XST",
            Self::Suitcase => "XSU",
            Self::EnvelopeSteel => "XSV",
            Self::Shrinkwrapped => "XSW",
            Self::Set => "XSX",
            Self::Sleeve => "XSY",
            Self::SheetsInBundleBunchTruss => "XSZ",
            Self::Tablet => "XT1",
            Self::Tub => "XTB",
            Self::TeaChest => "XTC",
            Self::TubeCollapsible => "XTD",
            Self::Tyre => "XTE",
            Self::TankContainerGeneric => "XTG",
            Self::Tierce => "XTI",
            Self::TankRectangular => "XTK",
            Self::TubWithLid => "XTL",
            Self::Tin => "XTN",
            Self::Tun => "XTO",
            Self::Trunk => "XTR",
            Self::Truss => "XTS",
            Self::BagTote => "XTT",
            Self::Tube => "XTU",
            Self::TubeWithNozzle => "XTV",
            Self::PalletTriwall => "XTW",
            Self::TankCylindrical => "XTY",
            Self::TubesInBundleBunchTruss => "XTZ",
            Self::Uncaged => "XUC",
            Self::Unit => "XUN",
            Self::Vat => "XVA",
            Self::BulkGasAt1031MbarAnd15C => "XVG",
            Self::Vial => "XVI",
            Self::Vanpack => "XVK",
            Self::BulkLiquid => "XVL",
            Self::Vehicle => "XVN",
            Self::BulkSolidLargeParticlesNodules => "XVO",
            Self::VacuumPacked => "XVP",
            Self::BulkLiquefiedGasAtAbnormalTemperaturePressure => "XVQ",
            Self::BulkSolidGranularParticlesGrains => "XVR",
            Self::BulkScrapMetal => "XVS",
            Self::BulkSolidFineParticlesPowders => "XVY",
            Self::IntermediateBulkContainer => "XWA",
            Self::Wickerbottle => "XWB",
            Self::IntermediateBulkContainerSteel => "XWC",
            Self::IntermediateBulkContainerAluminium => "XWD",
            Self::IntermediateBulkContainerMetal => "XWF",
            Self::IntermediateBulkContainerSteelPressurised10Kpa => "XWG",
            Self::IntermediateBulkContainerAluminiumPressurised10Kpa => "XWH",
            Self::IntermediateBulkContainerMetalPressure10Kpa => "XWJ",
            Self::IntermediateBulkContainerSteelLiquid => "XWK",
            Self::IntermediateBulkContainerAluminiumLiquid => "XWL",
            Self::IntermediateBulkContainerMetalLiquid => "XWM",
            Self::IntermediateBulkContainerWovenPlasticWithoutCoatLiner => "XWN",
            Self::IntermediateBulkContainerWovenPlasticCoated => "XWP",
            Self::IntermediateBulkContainerWovenPlasticWithLiner => "XWQ",
            Self::IntermediateBulkContainerWovenPlasticCoatedAndLiner => "XWR",
            Self::IntermediateBulkContainerPlasticFilm => "XWS",
            Self::IntermediateBulkContainerTextileWithOutCoatLiner => "XWT",
            Self::IntermediateBulkContainerNaturalWoodWithInnerLiner => "XWU",
            Self::IntermediateBulkContainerTextileCoated => "XWV",
            Self::IntermediateBulkContainerTextileWithLiner => "XWW",
            Self::IntermediateBulkContainerTextileCoatedAndLiner => "XWX",
            Self::IntermediateBulkContainerPlywoodWithInnerLiner => "XWY",
            Self::IntermediateBulkContainerReconstitutedWoodWithInnerLiner => "XWZ",
            Self::BagWovenPlasticWithoutInnerCoatLiner => "XXA",
            Self::BagWovenPlasticSiftProof => "XXB",
            Self::BagWovenPlasticWaterResistant => "XXC",
            Self::BagPlasticsFilm => "XXD",
            Self::BagTextileWithoutInnerCoatLiner => "XXF",
            Self::BagTextileSiftProof => "XXG",
            Self::BagTextileWaterResistant => "XXH",
            Self::BagPaperMultiWall => "XXJ",
            Self::BagPaperMultiWallWaterResistant => "XXK",
            Self::CompositePackagingPlasticReceptacleInSteelDrum => "XYA",
            Self::CompositePackagingPlasticReceptacleInSteelCrateBox => "XYB",
            Self::CompositePackagingPlasticReceptacleInAluminiumDrum => "XYC",
            Self::CompositePackagingPlasticReceptacleInAluminiumCrate => "XYD",
            Self::CompositePackagingPlasticReceptacleInWoodenBox => "XYF",
            Self::CompositePackagingPlasticReceptacleInPlywoodDrum => "XYG",
            Self::CompositePackagingPlasticReceptacleInPlywoodBox => "XYH",
            Self::CompositePackagingPlasticReceptacleInFibreDrum => "XYJ",
            Self::CompositePackagingPlasticReceptacleInFibreboardBox => "XYK",
            Self::CompositePackagingPlasticReceptacleInPlasticDrum => "XYL",
            Self::CompositePackagingPlasticReceptacleInSolidPlasticBox => "XYM",
            Self::CompositePackagingGlassReceptacleInSteelDrum => "XYN",
            Self::CompositePackagingGlassReceptacleInSteelCrateBox => "XYP",
            Self::CompositePackagingGlassReceptacleInAluminiumDrum => "XYQ",
            Self::CompositePackagingGlassReceptacleInAluminiumCrate => "XYR",
            Self::CompositePackagingGlassReceptacleInWoodenBox => "XYS",
            Self::CompositePackagingGlassReceptacleInPlywoodDrum => "XYT",
            Self::CompositePackagingGlassReceptacleInWickerworkHamper => "XYV",
            Self::CompositePackagingGlassReceptacleInFibreDrum => "XYW",
            Self::CompositePackagingGlassReceptacleInFibreboardBox => "XYX",
            Self::CompositePackagingGlassReceptacleInExpandablePlasticPack => "XYY",
            Self::CompositePackagingGlassReceptacleInSolidPlasticPack => "XYZ",
            Self::IntermediateBulkContainerPaperMultiWall => "XZA",
            Self::BagLarge => "XZB",
            Self::IntermediateBulkContainerPaperMultiWallWaterResistant => "XZC",
            Self::IntermediateBulkContainerRigidPlasticWithStructuralEquipmentSolids => "XZD",
            Self::IntermediateBulkContainerRigidPlasticFreestandingSolids => "XZF",
            Self::IntermediateBulkContainerRigidPlasticWithStructuralEquipmentPressurised => "XZG",
            Self::IntermediateBulkContainerRigidPlasticFreestandingPressurised => "XZH",
            Self::IntermediateBulkContainerRigidPlasticWithStructuralEquipmentLiquids => "XZJ",
            Self::IntermediateBulkContainerRigidPlasticFreestandingLiquids => "XZK",
            Self::IntermediateBulkContainerCompositeRigidPlasticSolids => "XZL",
            Self::IntermediateBulkContainerCompositeFlexiblePlasticSolids => "XZM",
            Self::IntermediateBulkContainerCompositeRigidPlasticPressurised => "XZN",
            Self::IntermediateBulkContainerCompositeFlexiblePlasticPressurised => "XZP",
            Self::IntermediateBulkContainerCompositeRigidPlasticLiquids => "XZQ",
            Self::IntermediateBulkContainerCompositeFlexiblePlasticLiquids => "XZR",
            Self::IntermediateBulkContainerComposite => "XZS",
            Self::IntermediateBulkContainerFibreboard => "XZT",
            Self::IntermediateBulkContainerFlexible => "XZU",
            Self::IntermediateBulkContainerMetalOtherThanSteel => "XZV",
            Self::IntermediateBulkContainerNaturalWood => "XZW",
            Self::IntermediateBulkContainerPlywood => "XZX",
            Self::IntermediateBulkContainerReconstitutedWood => "XZY",
            Self::MutuallyDefined => "XZZ",
        }
    }

    /// The human-readable name of this package type.
    pub fn name(self) -> &'static str {
        match self {
            Self::X1 => "X1",
            Self::DrumSteel => "Drum, steel",
            Self::DrumAluminium => "Drum, aluminium",
            Self::DrumPlywood => "Drum, plywood",
            Self::ContainerFlexible => "Container, flexible",
            Self::DrumFibre => "Drum, fibre",
            Self::DrumWooden => "Drum, wooden",
            Self::BarrelWooden => "Barrel, wooden",
            Self::JerricanSteel => "Jerrican, steel",
            Self::JerricanPlastic => "Jerrican, plastic",
            Self::BagSuperBulk => "Bag, super bulk",
            Self::BagPolybag => "Bag, polybag",
            Self::BoxSteel => "Box, steel",
            Self::BoxAluminium => "Box, aluminium",
            Self::BoxNaturalWood => "Box, natural wood",
            Self::BoxPlywood => "Box, plywood",
            Self::BoxReconstitutedWood => "Box, reconstituted wood",
            Self::BoxFibreboard => "Box, fibreboard",
            Self::BoxPlastic => "Box, plastic",
            Self::BagWovenPlastic => "Bag, woven plastic",
            Self::BagTextile => "Bag, textile",
            Self::BagPaper => "Bag, paper",
            Self::CompositePackagingPlasticReceptacle => "Composite packaging, plastic receptacle",
            Self::CompositePackagingGlassReceptacle => "Composite packaging, glass receptacle",
            Self::CaseCar => "Case, car",
            Self::CaseWooden => "Case, wooden",
            Self::PalletWooden => "Pallet, wooden",
            Self::CrateWooden => "Crate, wooden",
            Self::BundleWooden => "Bundle, wooden",
            Self::IntermediateBulkContainerRigidPlastic => {
                "Intermediate bulk container, rigid plastic"
            }
            Self::ReceptacleFibre => "Receptacle, fibre",
            Self::ReceptaclePaper => "Receptacle, paper",
            Self::ReceptacleWooden => "Receptacle, wooden",
            Self::Aerosol => "Aerosol",
            Self::PalletModularCollars80cms60cms => "Pallet, modular, collars 80cms * 60cms",
            Self::PalletShrinkwrapped => "Pallet, shrinkwrapped",
            Self::Pallet100cms110cms => "Pallet, 100cms * 110cms",
            Self::Clamshell => "Clamshell",
            Self::Cone => "Cone",
            Self::Ball => "Ball",
            Self::AmpouleNonProtected => "Ampoule, non-protected",
            Self::AmpouleProtected => "Ampoule, protected",
            Self::Atomizer => "Atomizer",
            Self::Capsule => "Capsule",
            Self::Belt => "Belt",
            Self::Barrel => "Barrel",
            Self::Bobbin => "Bobbin",
            Self::BottlecrateBottlerack => "Bottlecrate / bottlerack",
            Self::Board => "Board",
            Self::Bundle => "Bundle",
            Self::BalloonNonProtected => "Balloon, non-protected",
            Self::Bag => "Bag",
            Self::Bunch => "Bunch",
            Self::Bin => "Bin",
            Self::Bucket => "Bucket",
            Self::Basket => "Basket",
            Self::BaleCompressed => "Bale, compressed",
            Self::Basin => "Basin",
            Self::BaleNonCompressed => "Bale, non-compressed",
            Self::BottleNonProtectedCylindrical => "Bottle, non-protected, cylindrical",
            Self::BalloonProtected => "Balloon, protected",
            Self::BottleProtectedCylindrical => "Bottle, protected cylindrical",
            Self::Bar => "Bar",
            Self::BottleNonProtectedBulbous => "Bottle, non-protected, bulbous",
            Self::Bolt => "Bolt",
            Self::Butt => "Butt",
            Self::BottleProtectedBulbous => "Bottle, protected bulbous",
            Self::BoxForLiquids => "Box, for liquids",
            Self::Box => "Box",
            Self::BoardInBundleBunchTruss => "Board, in bundle/bunch/truss",
            Self::BarsInBundleBunchTruss => "Bars, in bundle/bunch/truss",
            Self::CanRectangular => "Can, rectangular",
            Self::CrateBeer => "Crate, beer",
            Self::Churn => "Churn",
            Self::CanWithHandleAndSpout => "Can, with handle and spout",
            Self::Creel => "Creel",
            Self::Coffer => "Coffer",
            Self::Cage => "Cage",
            Self::Chest => "Chest",
            Self::Canister => "Canister",
            Self::Coffin => "Coffin",
            Self::Cask => "Cask",
            Self::Coil => "Coil",
            Self::Card => "Card",
            Self::ContainerNotOtherwiseSpecifiedAsTransportEquipment => {
                "Container, not otherwise specified as transport equipment"
            }
            Self::CarboyNonProtected => "Carboy, non-protected",
            Self::CarboyProtected => "Carboy, protected",
            Self::Cartridge => "Cartridge",
            Self::Crate => "Crate",
            Self::Case => "Case",
            Self::Carton => "Carton",
            Self::Cup => "Cup",
            Self::Cover => "Cover",
            Self::CageRoll => "Cage, roll",
            Self::CanCylindrical => "Can, cylindrical",
            Self::Cylinder => "Cylinder",
            Self::Canvas => "Canvas",
            Self::CrateMultipleLayerPlastic => "Crate, multiple layer, plastic",
            Self::CrateMultipleLayerWooden => "Crate, multiple layer, wooden",
            Self::CrateMultipleLayerCardboard => "Crate, multiple layer, cardboard",
            Self::CageCommonwealthHandlingEquipmentPoolCHEP => {
                "Cage, Commonwealth Handling Equipment Pool  (CHEP)"
            }
            Self::BoxCommonwealthHandlingEquipmentPoolCHEPEurobox => {
                "Box, Commonwealth Handling Equipment Pool (CHEP), Eurobox"
            }
            Self::DrumIron => "Drum, iron",
            Self::DemijohnNonProtected => "Demijohn, non-protected",
            Self::CrateBulkCardboard => "Crate, bulk, cardboard",
            Self::CrateBulkPlastic => "Crate, bulk, plastic",
            Self::CrateBulkWooden => "Crate, bulk, wooden",
            Self::Dispenser => "Dispenser",
            Self::DemijohnProtected => "Demijohn, protected",
            Self::Drum => "Drum",
            Self::TrayOneLayerNoCoverPlastic => "Tray, one layer no cover, plastic",
            Self::TrayOneLayerNoCoverWooden => "Tray, one layer no cover, wooden",
            Self::TrayOneLayerNoCoverPolystyrene => "Tray, one layer no cover, polystyrene",
            Self::TrayOneLayerNoCoverCardboard => "Tray, one layer no cover, cardboard",
            Self::TrayTwoLayersNoCoverPlasticTray => "Tray, two layers no cover, plastic tray",
            Self::TrayTwoLayersNoCoverWooden => "Tray, two layers no cover, wooden",
            Self::TrayTwoLayersNoCoverCardboard => "Tray, two layers no cover, cardboard",
            Self::BagPlastic => "Bag, plastic",
            Self::CaseWithPalletBase => "Case, with pallet base",
            Self::CaseWithPalletBaseWooden => "Case, with pallet base, wooden",
            Self::CaseWithPalletBaseCardboard => "Case, with pallet base, cardboard",
            Self::CaseWithPalletBasePlastic => "Case, with pallet base, plastic",
            Self::CaseWithPalletBaseMetal => "Case, with pallet base, metal",
            Self::CaseIsothermic => "Case, isothermic",
            Self::Envelope => "Envelope",
            Self::Flexibag => "Flexibag",
            Self::CrateFruit => "Crate, fruit",
            Self::CrateFramed => "Crate, framed",
            Self::Flexitank => "Flexitank",
            Self::Firkin => "Firkin",
            Self::Flask => "Flask",
            Self::Footlocker => "Footlocker",
            Self::Filmpack => "Filmpack",
            Self::Frame => "Frame",
            Self::Foodtainer => "Foodtainer",
            Self::CartFlatbed => "Cart, flatbed",
            Self::BagFlexibleContainer => "Bag, flexible container",
            Self::BottleGas => "Bottle, gas",
            Self::Girder => "Girder",
            Self::ContainerGallon => "Container, gallon",
            Self::ReceptacleGlass => "Receptacle, glass",
            Self::TrayContainingHorizontallyStackedFlatItems => {
                "Tray, containing horizontally stacked flat items"
            }
            Self::BagGunny => "Bag, gunny",
            Self::GirdersInBundleBunchTruss => "Girders, in bundle/bunch/truss",
            Self::BasketWithHandlePlastic => "Basket, with handle, plastic",
            Self::BasketWithHandleWooden => "Basket, with handle, wooden",
            Self::BasketWithHandleCardboard => "Basket, with handle, cardboard",
            Self::Hogshead => "Hogshead",
            Self::Hanger => "Hanger",
            Self::Hamper => "Hamper",
            Self::PackageDisplayWooden => "Package, display, wooden",
            Self::PackageDisplayCardboard => "Package, display, cardboard",
            Self::PackageDisplayPlastic => "Package, display, plastic",
            Self::PackageDisplayMetal => "Package, display, metal",
            Self::PackageShow => "Package, show",
            Self::PackageFlow => "Package, flow",
            Self::PackagePaperWrapped => "Package, paper wrapped",
            Self::DrumPlastic => "Drum, plastic",
            Self::PackageCardboardWithBottleGripHoles => {
                "Package, cardboard, with bottle grip-holes"
            }
            Self::TrayRigidLiddedStackableCENTS144822002 => {
                "Tray, rigid, lidded stackable (CEN TS 14482:2002)"
            }
            Self::Ingot => "Ingot",
            Self::IngotsInBundleBunchTruss => "Ingots, in bundle/bunch/truss",
            Self::BagJumbo => "Bag, jumbo",
            Self::JerricanRectangular => "Jerrican, rectangular",
            Self::Jug => "Jug",
            Self::Jar => "Jar",
            Self::Jutebag => "Jutebag",
            Self::JerricanCylindrical => "Jerrican, cylindrical",
            Self::Keg => "Keg",
            Self::Kit => "Kit",
            Self::Luggage => "Luggage",
            Self::Log => "Log",
            Self::Lot => "Lot",
            Self::Lug => "Lug",
            Self::Liftvan => "Liftvan",
            Self::LogsInBundleBunchTruss => "Logs, in bundle/bunch/truss",
            Self::CrateMetal => "Crate, metal",
            Self::BagMultiply => "Bag, multiply",
            Self::CrateMilk => "Crate, milk",
            Self::ContainerMetal => "Container, metal",
            Self::ReceptacleMetal => "Receptacle, metal",
            Self::SackMultiWall => "Sack, multi-wall",
            Self::Mat => "Mat",
            Self::ReceptaclePlasticWrapped => "Receptacle, plastic wrapped",
            Self::Matchbox => "Matchbox",
            Self::NotAvailable => "Not available",
            Self::UnpackedOrUnpackaged => "Unpacked or unpackaged",
            Self::UnpackedOrUnpackagedSingleUnit => "Unpacked or unpackaged, single unit",
            Self::UnpackedOrUnpackagedMultipleUnits => "Unpacked or unpackaged, multiple units",
            Self::Nest => "Nest",
            Self::Net => "Net",
            Self::NetTubePlastic => "Net, tube, plastic",
            Self::NetTubeTextile => "Net, tube, textile",
            Self::TwoSidedCageOnWheelsWithFixingStrap => {
                "Two sided cage on wheels with fixing strap"
            }
            Self::Trolley => "Trolley",
            Self::OnewayPalletISO012EUROPallet => "Oneway pallet ISO 0 - 1/2 EURO Pallet",
            Self::OnewayPalletISO111EUROPallet => "Oneway pallet ISO 1 - 1/1 EURO Pallet",
            Self::OnewayPalletISO221EUROPallet => "Oneway pallet ISO 2 - 2/1 EURO Pallet",
            Self::PalletWithExceptionalDimensions => "Pallet with exceptional dimensions",
            Self::WoodenPallet40CmX80Cm => "Wooden pallet  40 cm x 80 cm",
            Self::PlasticPalletSRS60CmX80Cm => "Plastic pallet SRS 60 cm x 80 cm",
            Self::PlasticPalletSRS80CmX120Cm => "Plastic pallet SRS 80 cm x 120 cm",
            Self::PalletCHEP40CmX60Cm => "Pallet, CHEP 40 cm x 60 cm",
            Self::PalletCHEP80CmX120Cm => "Pallet, CHEP 80 cm x 120 cm",
            Self::PalletCHEP100CmX120Cm => "Pallet, CHEP 100 cm x 120 cm",
            Self::PalletAS40681993 => "Pallet, AS 4068-1993",
            Self::PalletISOT11 => "Pallet, ISO T11",
            Self::PlatformUnspecifiedWeightOrDimension => {
                "Platform, unspecified weight or dimension"
            }
            Self::PalletISO012EUROPallet => "Pallet ISO 0 - 1/2 EURO Pallet",
            Self::PalletISO111EUROPallet => "Pallet ISO 1 - 1/1 EURO Pallet",
            Self::PalletISO221EUROPallet => "Pallet ISO 2 – 2/1 EURO Pallet",
            Self::P14EUROPallet => "1/4 EURO Pallet",
            Self::Block => "Block",
            Self::P18EUROPallet => "1/8 EURO Pallet",
            Self::SyntheticPalletISO1 => "Synthetic pallet ISO 1",
            Self::SyntheticPalletISO2 => "Synthetic pallet ISO 2",
            Self::WholesalerPallet => "Wholesaler pallet",
            Self::Pallet80X100Cm => "Pallet 80 X 100 cm",
            Self::Pallet60X100Cm => "Pallet 60 X 100 cm",
            Self::OnewayPallet => "Oneway pallet",
            Self::Octabin => "Octabin",
            Self::ContainerOuter => "Container, outer",
            Self::ReturnablePallet => "Returnable pallet",
            Self::LargeBagPalletSized => "Large bag, pallet sized",
            Self::AWheeledPalletWithRaisedRim81X67X135 => {
                "A wheeled pallet with raised rim (81 x 67 x 135)"
            }
            Self::AWheeledPalletWithRaisedRim81X72X135 => {
                "A Wheeled pallet with raised rim (81 x 72 x 135)"
            }
            Self::WheeledPalletWithRaisedRim81X60X16 => {
                "Wheeled pallet with raised rim ( 81 x 60 x 16)"
            }
            Self::CHEPPallet60CmX80Cm => "CHEP pallet 60 cm x 80 cm",
            Self::Pan => "Pan",
            Self::LPRPallet60CmX80Cm => "LPR pallet 60 cm x 80 cm",
            Self::LPRPallet80CmX120Cm => "LPR pallet 80 cm x 120 cm",
            Self::Packet => "Packet",
            Self::PalletBoxCombinedOpenEndedBoxAndPallet => {
                "Pallet, box Combined open-ended box and pallet"
            }
            Self::Parcel => "Parcel",
            Self::PalletModularCollars80cms100cms => "Pallet, modular, collars 80cms * 100cms",
            Self::PalletModularCollars80cms120cms => "Pallet, modular, collars 80cms * 120cms",
            Self::Pen => "Pen",
            Self::Plate => "Plate",
            Self::Pitcher => "Pitcher",
            Self::Pipe => "Pipe",
            Self::Punnet => "Punnet",
            Self::Package => "Package",
            Self::Pail => "Pail",
            Self::Plank => "Plank",
            Self::Pouch => "Pouch",
            Self::Piece => "Piece",
            Self::ReceptaclePlastic => "Receptacle, plastic",
            Self::Pot => "Pot",
            Self::Tray => "Tray",
            Self::PipesInBundleBunchTruss => "Pipes, in bundle/bunch/truss",
            Self::Pallet => "Pallet",
            Self::PlatesInBundleBunchTruss => "Plates, in bundle/bunch/truss",
            Self::PlanksInBundleBunchTruss => "Planks, in bundle/bunch/truss",
            Self::DrumSteelNonRemovableHead => "Drum, steel, non-removable head",
            Self::DrumSteelRemovableHead => "Drum, steel, removable head",
            Self::DrumAluminiumNonRemovableHead => "Drum, aluminium, non-removable head",
            Self::DrumAluminiumRemovableHead => "Drum, aluminium, removable head",
            Self::DrumPlasticNonRemovableHead => "Drum, plastic, non-removable head",
            Self::DrumPlasticRemovableHead => "Drum, plastic, removable head",
            Self::BarrelWoodenBungType => "Barrel, wooden, bung type",
            Self::BarrelWoodenRemovableHead => "Barrel, wooden, removable head",
            Self::JerricanSteelNonRemovableHead => "Jerrican, steel, non-removable head",
            Self::JerricanSteelRemovableHead => "Jerrican, steel, removable head",
            Self::JerricanPlasticNonRemovableHead => "Jerrican, plastic, non-removable head",
            Self::JerricanPlasticRemovableHead => "Jerrican, plastic, removable head",
            Self::BoxWoodenNaturalWoodOrdinary => "Box, wooden, natural wood, ordinary",
            Self::BoxWoodenNaturalWoodWithSiftProofWalls => {
                "Box, wooden, natural wood, with sift proof walls"
            }
            Self::BoxPlasticExpanded => "Box, plastic, expanded",
            Self::BoxPlasticSolid => "Box, plastic, solid",
            Self::Rod => "Rod",
            Self::Ring => "Ring",
            Self::RackClothingHanger => "Rack, clothing hanger",
            Self::Rack => "Rack",
            Self::Reel => "Reel",
            Self::Roll => "Roll",
            Self::Rednet => "Rednet",
            Self::RodsInBundleBunchTruss => "Rods, in bundle/bunch/truss",
            Self::Sack => "Sack",
            Self::Slab => "Slab",
            Self::CrateShallow => "Crate, shallow",
            Self::Spindle => "Spindle",
            Self::SeaChest => "Sea-chest",
            Self::Sachet => "Sachet",
            Self::Skid => "Skid",
            Self::CaseSkeleton => "Case, skeleton",
            Self::Slipsheet => "Slipsheet",
            Self::Sheetmetal => "Sheetmetal",
            Self::Spool => "Spool",
            Self::SheetPlasticWrapping => "Sheet, plastic wrapping",
            Self::CaseSteel => "Case, steel",
            Self::Sheet => "Sheet",
            Self::Suitcase => "Suitcase",
            Self::EnvelopeSteel => "Envelope, steel",
            Self::Shrinkwrapped => "Shrinkwrapped",
            Self::Set => "Set",
            Self::Sleeve => "Sleeve",
            Self::SheetsInBundleBunchTruss => "Sheets, in bundle/bunch/truss",
            Self::Tablet => "Tablet",
            Self::Tub => "Tub",
            Self::TeaChest => "Tea-chest",
            Self::TubeCollapsible => "Tube, collapsible",
            Self::Tyre => "Tyre",
            Self::TankContainerGeneric => "Tank container, generic",
            Self::Tierce => "Tierce",
            Self::TankRectangular => "Tank, rectangular",
            Self::TubWithLid => "Tub, with lid",
            Self::Tin => "Tin",
            Self::Tun => "Tun",
            Self::Trunk => "Trunk",
            Self::Truss => "Truss",
            Self::BagTote => "Bag, tote",
            Self::Tube => "Tube",
            Self::TubeWithNozzle => "Tube, with nozzle",
            Self::PalletTriwall => "Pallet, triwall",
            Self::TankCylindrical => "Tank, cylindrical",
            Self::TubesInBundleBunchTruss => "Tubes, in bundle/bunch/truss",
            Self::Uncaged => "Uncaged",
            Self::Unit => "Unit",
            Self::Vat => "Vat",
            Self::BulkGasAt1031MbarAnd15C => "Bulk, gas (at 1031 mbar and 15°C)",
            Self::Vial => "Vial",
            Self::Vanpack => "Vanpack",
            Self::BulkLiquid => "Bulk, liquid",
            Self::Vehicle => "Vehicle",
            Self::BulkSolidLargeParticlesNodules => "Bulk, solid, large particles (“nodules”)",
            Self::VacuumPacked => "Vacuum-packed",
            Self::BulkLiquefiedGasAtAbnormalTemperaturePressure => {
                "Bulk, liquefied gas (at abnormal temperature/pressure)"
            }
            Self::BulkSolidGranularParticlesGrains => "Bulk, solid, granular particles (“grains”)",
            Self::BulkScrapMetal => "Bulk, scrap metal",
            Self::BulkSolidFineParticlesPowders => "Bulk, solid, fine particles (“powders”)",
            Self::IntermediateBulkContainer => "Intermediate bulk container",
            Self::Wickerbottle => "Wickerbottle",
            Self::IntermediateBulkContainerSteel => "Intermediate bulk container, steel",
            Self::IntermediateBulkContainerAluminium => "Intermediate bulk container, aluminium",
            Self::IntermediateBulkContainerMetal => "Intermediate bulk container, metal",
            Self::IntermediateBulkContainerSteelPressurised10Kpa => {
                "Intermediate bulk container, steel, pressurised > 10 kpa"
            }
            Self::IntermediateBulkContainerAluminiumPressurised10Kpa => {
                "Intermediate bulk container, aluminium, pressurised > 10 kpa"
            }
            Self::IntermediateBulkContainerMetalPressure10Kpa => {
                "Intermediate bulk container, metal, pressure 10 kpa"
            }
            Self::IntermediateBulkContainerSteelLiquid => {
                "Intermediate bulk container, steel, liquid"
            }
            Self::IntermediateBulkContainerAluminiumLiquid => {
                "Intermediate bulk container, aluminium, liquid"
            }
            Self::IntermediateBulkContainerMetalLiquid => {
                "Intermediate bulk container, metal, liquid"
            }
            Self::IntermediateBulkContainerWovenPlasticWithoutCoatLiner => {
                "Intermediate bulk container, woven plastic, without coat/liner"
            }
            Self::IntermediateBulkContainerWovenPlasticCoated => {
                "Intermediate bulk container, woven plastic, coated"
            }
            Self::IntermediateBulkContainerWovenPlasticWithLiner => {
                "Intermediate bulk container, woven plastic, with liner"
            }
            Self::IntermediateBulkContainerWovenPlasticCoatedAndLiner => {
                "Intermediate bulk container, woven plastic, coated and liner"
            }
            Self::IntermediateBulkContainerPlasticFilm => {
                "Intermediate bulk container, plastic film"
            }
            Self::IntermediateBulkContainerTextileWithOutCoatLiner => {
                "Intermediate bulk container, textile with out coat/liner"
            }
            Self::IntermediateBulkContainerNaturalWoodWithInnerLiner => {
                "Intermediate bulk container, natural wood, with inner liner"
            }
            Self::IntermediateBulkContainerTextileCoated => {
                "Intermediate bulk container, textile, coated"
            }
            Self::IntermediateBulkContainerTextileWithLiner => {
                "Intermediate bulk container, textile, with liner"
            }
            Self::IntermediateBulkContainerTextileCoatedAndLiner => {
                "Intermediate bulk container, textile, coated and liner"
            }
            Self::IntermediateBulkContainerPlywoodWithInnerLiner => {
                "Intermediate bulk container, plywood, with inner liner"
            }
            Self::IntermediateBulkContainerReconstitutedWoodWithInnerLiner => {
                "Intermediate bulk container, reconstituted wood, with inner liner"
            }
            Self::BagWovenPlasticWithoutInnerCoatLiner => {
                "Bag, woven plastic, without inner coat/liner"
            }
            Self::BagWovenPlasticSiftProof => "Bag, woven plastic, sift proof",
            Self::BagWovenPlasticWaterResistant => "Bag, woven plastic, water resistant",
            Self::BagPlasticsFilm => "Bag, plastics film",
            Self::BagTextileWithoutInnerCoatLiner => "Bag, textile, without inner coat/liner",
            Self::BagTextileSiftProof => "Bag, textile, sift proof",
            Self::BagTextileWaterResistant => "Bag, textile, water resistant",
            Self::BagPaperMultiWall => "Bag, paper, multi-wall",
            Self::BagPaperMultiWallWaterResistant => "Bag, paper, multi-wall, water resistant",
            Self::CompositePackagingPlasticReceptacleInSteelDrum => {
                "Composite packaging, plastic receptacle in steel drum"
            }
            Self::CompositePackagingPlasticReceptacleInSteelCrateBox => {
                "Composite packaging, plastic receptacle in steel crate box"
            }
            Self::CompositePackagingPlasticReceptacleInAluminiumDrum => {
                "Composite packaging, plastic receptacle in aluminium drum"
            }
            Self::CompositePackagingPlasticReceptacleInAluminiumCrate => {
                "Composite packaging, plastic receptacle in aluminium crate"
            }
            Self::CompositePackagingPlasticReceptacleInWoodenBox => {
                "Composite packaging, plastic receptacle in wooden box"
            }
            Self::CompositePackagingPlasticReceptacleInPlywoodDrum => {
                "Composite packaging, plastic receptacle in plywood drum"
            }
            Self::CompositePackagingPlasticReceptacleInPlywoodBox => {
                "Composite packaging, plastic receptacle in plywood box"
            }
            Self::CompositePackagingPlasticReceptacleInFibreDrum => {
                "Composite packaging, plastic receptacle in fibre drum"
            }
            Self::CompositePackagingPlasticReceptacleInFibreboardBox => {
                "Composite packaging, plastic receptacle in fibreboard box"
            }
            Self::CompositePackagingPlasticReceptacleInPlasticDrum => {
                "Composite packaging, plastic receptacle in plastic drum"
            }
            Self::CompositePackagingPlasticReceptacleInSolidPlasticBox => {
                "Composite packaging, plastic receptacle in solid plastic box"
            }
            Self::CompositePackagingGlassReceptacleInSteelDrum => {
                "Composite packaging, glass receptacle in steel drum"
            }
            Self::CompositePackagingGlassReceptacleInSteelCrateBox => {
                "Composite packaging, glass receptacle in steel crate box"
            }
            Self::CompositePackagingGlassReceptacleInAluminiumDrum => {
                "Composite packaging, glass receptacle in aluminium drum"
            }
            Self::CompositePackagingGlassReceptacleInAluminiumCrate => {
                "Composite packaging, glass receptacle in aluminium crate"
            }
            Self::CompositePackagingGlassReceptacleInWoodenBox => {
                "Composite packaging, glass receptacle in wooden box"
            }
            Self::CompositePackagingGlassReceptacleInPlywoodDrum => {
                "Composite packaging, glass receptacle in plywood drum"
            }
            Self::CompositePackagingGlassReceptacleInWickerworkHamper => {
                "Composite packaging, glass receptacle in wickerwork hamper"
            }
            Self::CompositePackagingGlassReceptacleInFibreDrum => {
                "Composite packaging, glass receptacle in fibre drum"
            }
            Self::CompositePackagingGlassReceptacleInFibreboardBox => {
                "Composite packaging, glass receptacle in fibreboard box"
            }
            Self::CompositePackagingGlassReceptacleInExpandablePlasticPack => {
                "Composite packaging, glass receptacle in expandable plastic pack"
            }
            Self::CompositePackagingGlassReceptacleInSolidPlasticPack => {
                "Composite packaging, glass receptacle in solid plastic pack"
            }
            Self::IntermediateBulkContainerPaperMultiWall => {
                "Intermediate bulk container, paper, multi-wall"
            }
            Self::BagLarge => "Bag, large",
            Self::IntermediateBulkContainerPaperMultiWallWaterResistant => {
                "Intermediate bulk container, paper, multi-wall, water resistant"
            }
            Self::IntermediateBulkContainerRigidPlasticWithStructuralEquipmentSolids => {
                "Intermediate bulk container, rigid plastic, with structural equipment, solids"
            }
            Self::IntermediateBulkContainerRigidPlasticFreestandingSolids => {
                "Intermediate bulk container, rigid plastic, freestanding, solids"
            }
            Self::IntermediateBulkContainerRigidPlasticWithStructuralEquipmentPressurised => {
                "Intermediate bulk container, rigid plastic, with structural equipment, pressurised"
            }
            Self::IntermediateBulkContainerRigidPlasticFreestandingPressurised => {
                "Intermediate bulk container, rigid plastic, freestanding, pressurised"
            }
            Self::IntermediateBulkContainerRigidPlasticWithStructuralEquipmentLiquids => {
                "Intermediate bulk container, rigid plastic, with structural equipment, liquids"
            }
            Self::IntermediateBulkContainerRigidPlasticFreestandingLiquids => {
                "Intermediate bulk container, rigid plastic, freestanding, liquids"
            }
            Self::IntermediateBulkContainerCompositeRigidPlasticSolids => {
                "Intermediate bulk container, composite, rigid plastic, solids"
            }
            Self::IntermediateBulkContainerCompositeFlexiblePlasticSolids => {
                "Intermediate bulk container, composite, flexible plastic, solids"
            }
            Self::IntermediateBulkContainerCompositeRigidPlasticPressurised => {
                "Intermediate bulk container, composite, rigid plastic, pressurised"
            }
            Self::IntermediateBulkContainerCompositeFlexiblePlasticPressurised => {
                "Intermediate bulk container, composite, flexible plastic, pressurised"
            }
            Self::IntermediateBulkContainerCompositeRigidPlasticLiquids => {
                "Intermediate bulk container, composite, rigid plastic, liquids"
            }
            Self::IntermediateBulkContainerCompositeFlexiblePlasticLiquids => {
                "Intermediate bulk container, composite, flexible plastic, liquids"
            }
            Self::IntermediateBulkContainerComposite => "Intermediate bulk container, composite",
            Self::IntermediateBulkContainerFibreboard => "Intermediate bulk container, fibreboard",
            Self::IntermediateBulkContainerFlexible => "Intermediate bulk container, flexible",
            Self::IntermediateBulkContainerMetalOtherThanSteel => {
                "Intermediate bulk container, metal, other than steel"
            }
            Self::IntermediateBulkContainerNaturalWood => {
                "Intermediate bulk container, natural wood"
            }
            Self::IntermediateBulkContainerPlywood => "Intermediate bulk container, plywood",
            Self::IntermediateBulkContainerReconstitutedWood => {
                "Intermediate bulk container, reconstituted wood"
            }
            Self::MutuallyDefined => "Mutually defined",
        }
    }
}

impl FromStr for PackageCode {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::from_code(value).ok_or_else(|| Error::InvalidValue(format!("{value:?}")))
    }
}

impl TryFrom<&str> for PackageCode {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        <Self as FromStr>::from_str(value)
    }
}

impl Display for PackageCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.code().fmt(f)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_a_known_code() {
        let pkg: PackageCode = "XBA".parse().expect("XBA is a valid package code");

        assert_eq!(pkg.code(), "XBA");
        assert_eq!(pkg.name(), "Barrel");
    }

    #[test]
    fn rejects_an_unknown_code() {
        assert!("XZZZ".parse::<PackageCode>().is_err());
    }
}
