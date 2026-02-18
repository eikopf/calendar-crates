//! Types for finite, extensible set values.

/// A location type from the [IANA Location Types Registry].
///
/// [IANA Location Types Registry]: https://www.iana.org/assignments/location-type-registry/
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum LocationType {
    /// A device used for flight (airplane, helicopter, glider, balloon, etc.).
    ///
    /// Source: RFC 4589.
    Aircraft,
    /// A place from which aircraft operate (airport, heliport).
    ///
    /// Source: RFC 4589.
    Airport,
    /// Enclosed area used for sports events.
    ///
    /// Source: RFC 4589.
    Arena,
    /// An automotive vehicle designed for passenger transportation.
    ///
    /// Source: RFC 4589.
    Automobile,
    /// Business establishment for financial services.
    ///
    /// Source: RFC 4589.
    Bank,
    /// A bar or saloon.
    ///
    /// Source: RFC 4589.
    Bar,
    /// A two-wheeled pedal-propelled vehicle.
    ///
    /// Source: RFC 4589.
    Bicycle,
    /// A large motor vehicle designed to carry passengers.
    ///
    /// Source: RFC 4589.
    Bus,
    /// Terminal that serves bus passengers (bus depot, bus terminal).
    ///
    /// Source: RFC 4589.
    BusStation,
    /// A small informal establishment serving refreshments; coffee shop.
    ///
    /// Source: RFC 4589.
    Cafe,
    /// An area used for camping, often with facilities for tents or RVs.
    ///
    /// Source: NH Division of Emergency Services and Communications.
    Campground,
    /// A facility providing care services (nursing home, assisted living).
    ///
    /// Source: NH Division of Emergency Services and Communications.
    CareFacility,
    /// Academic classroom or lecture hall.
    ///
    /// Source: RFC 4589.
    Classroom,
    /// Dance club, nightclub, or discotheque.
    ///
    /// Source: RFC 4589.
    Club,
    /// Construction site.
    ///
    /// Source: RFC 4589.
    Construction,
    /// Convention center or exhibition hall.
    ///
    /// Source: RFC 4589.
    ConventionCenter,
    /// A building or housing unit that is not attached to other buildings.
    ///
    /// Source: NH Division of Emergency Services and Communications.
    DetachedUnit,
    /// A facility housing firefighting equipment and personnel.
    ///
    /// Source: NH Division of Emergency Services and Communications.
    FireStation,
    /// Government building (legislative, executive, judicial, police, military).
    ///
    /// Source: RFC 4589.
    Government,
    /// Hospital, hospice, medical clinic, mental institution, or doctor's office.
    ///
    /// Source: RFC 4589.
    Hospital,
    /// Hotel, motel, inn, or other lodging establishment.
    ///
    /// Source: RFC 4589.
    Hotel,
    /// Industrial setting (manufacturing floor, power plant).
    ///
    /// Source: RFC 4589.
    Industrial,
    /// A location identified by a landmark or notable address.
    ///
    /// Source: NH Division of Emergency Services and Communications.
    LandmarkAddress,
    /// Library or other public place for literary and artistic materials.
    ///
    /// Source: RFC 4589.
    Library,
    /// A two-wheeled automotive vehicle, including a scooter.
    ///
    /// Source: RFC 4589.
    Motorcycle,
    /// A garage owned or operated by a municipality.
    ///
    /// Source: NH Division of Emergency Services and Communications.
    MunicipalGarage,
    /// A building housing collections of artifacts or specimens.
    ///
    /// Source: NENA-STA-004.1.1-2025, NG9-1-1 CLDXF-US.
    Museum,
    /// Business setting, such as an office.
    ///
    /// Source: RFC 4589.
    Office,
    /// A place without a registered place type representation.
    ///
    /// Source: RFC 4589.
    Other,
    /// Outside a building, in the open air (park, city streets).
    ///
    /// Source: RFC 4589.
    Outdoors,
    /// A parking lot or parking garage.
    ///
    /// Source: RFC 4589.
    Parking,
    /// A public telephone booth or kiosk.
    ///
    /// Source: NH Division of Emergency Services and Communications.
    PhoneBox,
    /// A religious site (church, chapel, mosque, shrine, synagogue, temple).
    ///
    /// Source: RFC 4589.
    PlaceOfWorship,
    /// A post office or mail facility.
    ///
    /// Source: NH Division of Emergency Services and Communications.
    PostOffice,
    /// Correctional institution (prison, penitentiary, jail, brig).
    ///
    /// Source: RFC 4589.
    Prison,
    /// Public area (shopping mall, street, park, public building, conveyance).
    ///
    /// Source: RFC 4589.
    Public,
    /// Any form of public transport (aircraft, bus, train, ship).
    ///
    /// Source: RFC 4589.
    PublicTransport,
    /// A private or residential setting.
    ///
    /// Source: RFC 4589.
    Residence,
    /// Restaurant, coffee shop, or other public dining establishment.
    ///
    /// Source: RFC 4589.
    Restaurant,
    /// School or university property.
    ///
    /// Source: RFC 4589.
    School,
    /// Shopping mall or area with stores accessible by common passageways.
    ///
    /// Source: RFC 4589.
    ShoppingArea,
    /// Large structure for sports events, including a racetrack.
    ///
    /// Source: RFC 4589.
    Stadium,
    /// Place where merchandise is offered for sale; a shop.
    ///
    /// Source: RFC 4589.
    Store,
    /// A public thoroughfare (avenue, street, alley, lane, road, sidewalk).
    ///
    /// Source: RFC 4589.
    Street,
    /// Theater, auditorium, movie theater, or similar presentation facility.
    ///
    /// Source: RFC 4589.
    Theater,
    /// A booth or station for collecting tolls.
    ///
    /// Source: NH Division of Emergency Services and Communications.
    TollBooth,
    /// A building housing local government offices.
    ///
    /// Source: NH Division of Emergency Services and Communications.
    TownHall,
    /// Train, monorail, maglev, cable car, or similar conveyance.
    ///
    /// Source: RFC 4589.
    Train,
    /// Terminal where trains load or unload passengers or goods.
    ///
    /// Source: RFC 4589.
    TrainStation,
    /// An automotive vehicle for hauling goods rather than people.
    ///
    /// Source: RFC 4589.
    Truck,
    /// In a land-, water-, or aircraft that is in motion.
    ///
    /// Source: RFC 4589.
    Underway,
    /// The type of place is unknown.
    ///
    /// Source: RFC 4589.
    Unknown,
    /// A utility box or cabinet housing utility equipment.
    ///
    /// Source: NH Division of Emergency Services and Communications.
    UtilityBox,
    /// Place for storing goods (storehouse, self-storage facility).
    ///
    /// Source: RFC 4589.
    Warehouse,
    /// A facility for processing or transferring waste materials.
    ///
    /// Source: NH Division of Emergency Services and Communications.
    WasteTransferFacility,
    /// In, on, or above bodies of water (ocean, lake, river, canal).
    ///
    /// Source: RFC 4589.
    Water,
    /// A facility for water treatment or distribution.
    ///
    /// Source: NH Division of Emergency Services and Communications.
    WaterFacility,
    /// A vessel for travel on water (boat, ship).
    ///
    /// Source: RFC 4589.
    Watercraft,
    /// A camp providing programs and activities for youth.
    ///
    /// Source: NH Division of Emergency Services and Communications.
    YouthCamp,
}
