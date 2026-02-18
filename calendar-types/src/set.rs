//! Types for finite, extensible set values.

/// A link relation from the [IANA Link Relations Registry].
///
/// [IANA Link Relations Registry]: https://www.iana.org/assignments/link-relations/
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum LinkRelation {
    /// Refers to a resource that is the subject of the link's context.
    ///
    /// Source: RFC 6903, Section 2.
    About,
    /// Asserts that the link target provides an access control description.
    ///
    /// Source: Web Access Control.
    Acl,
    /// Refers to a substitute for this context.
    ///
    /// Source: HTML.
    Alternate,
    /// Refers to an AMP HTML version of the context.
    ///
    /// Source: AMP HTML.
    AmpHtml,
    /// Refers to a list of published APIs.
    ///
    /// Source: RFC 9727.
    ApiCatalog,
    /// Refers to an appendix in a collection of resources.
    ///
    /// Source: HTML 4.01.
    Appendix,
    /// Refers to an icon for the context (synonym for `icon`).
    ///
    /// Source: Apple Web Application Configuration.
    AppleTouchIcon,
    /// Refers to a launch screen for the context.
    ///
    /// Source: Apple Web Application Configuration.
    AppleTouchStartupImage,
    /// Refers to a collection of records, documents, or other materials.
    ///
    /// Source: HTML 5.
    Archives,
    /// Refers to the context's author.
    ///
    /// Source: HTML.
    Author,
    /// Identifies the entity that blocks access to a resource.
    ///
    /// Source: RFC 7725.
    BlockedBy,
    /// Gives a permanent link to use for bookmarking purposes.
    ///
    /// Source: HTML.
    Bookmark,
    /// Links to a C2PA Manifest associated with the link context.
    ///
    /// Source: C2PA Technical Specification.
    C2paManifest,
    /// Designates the preferred version of a resource.
    ///
    /// Source: RFC 6596.
    Canonical,
    /// Refers to a chapter in a collection of resources.
    ///
    /// Source: HTML 4.01.
    Chapter,
    /// Indicates the preferred URI for permanent citation.
    ///
    /// Source: RFC 8574.
    CiteAs,
    /// Represents a collection resource for the context.
    ///
    /// Source: RFC 6573.
    Collection,
    /// Refers to a compression dictionary for content encoding.
    ///
    /// Source: RFC 9842, Section 3.
    CompressionDictionary,
    /// Refers to a table of contents.
    ///
    /// Source: HTML 4.01.
    Contents,
    /// The document was later converted from the linked document.
    ///
    /// Source: RFC 7991.
    ConvertedFrom,
    /// Refers to a copyright statement for the context.
    ///
    /// Source: HTML 4.01.
    Copyright,
    /// Refers to a resource from which a submission form can be obtained.
    ///
    /// Source: RFC 6861.
    CreateForm,
    /// Refers to the most recent item(s) in a collection.
    ///
    /// Source: RFC 5005.
    Current,
    /// Links to documentation about deprecation of the context.
    ///
    /// Source: RFC 9745, Section 3.
    Deprecation,
    /// Refers to a resource providing information about the context.
    ///
    /// Source: Protocol for Web Description Resources (POWDER).
    DescribedBy,
    /// The relationship asserts that resource A describes resource B.
    ///
    /// Source: RFC 6892.
    Describes,
    /// Refers to a list of patent disclosures.
    ///
    /// Source: RFC 6579.
    Disclosure,
    /// Indicates an origin whose resources should be resolved early.
    ///
    /// Source: HTML.
    DnsPrefetch,
    /// Refers to a resource with byte-for-byte identical representation.
    ///
    /// Source: RFC 6249.
    Duplicate,
    /// Refers to a resource that can be used to edit the context.
    ///
    /// Source: RFC 5023.
    Edit,
    /// Refers to a submission form for editing the context.
    ///
    /// Source: RFC 6861.
    EditForm,
    /// Refers to a resource for editing media associated with the context.
    ///
    /// Source: RFC 5023.
    EditMedia,
    /// Identifies a potentially large related resource.
    ///
    /// Source: RFC 4287.
    Enclosure,
    /// Indicates a resource not part of the same site as the context.
    ///
    /// Source: HTML.
    External,
    /// Refers to the furthest preceding resource in a series.
    ///
    /// Source: RFC 8288.
    First,
    /// Refers to a resource providing IP geolocation information.
    ///
    /// Source: RFC 9877.
    Geofeed,
    /// Refers to a glossary of terms.
    ///
    /// Source: HTML 4.01.
    Glossary,
    /// Refers to context-sensitive help.
    ///
    /// Source: HTML.
    Help,
    /// Refers to a resource hosted by the server.
    ///
    /// Source: RFC 6690.
    Hosts,
    /// Refers to a hub for subscribing to updates.
    ///
    /// Source: WebSub.
    Hub,
    /// Refers to STUN/TURN server information for ICE connections.
    ///
    /// Source: RFC 9725.
    IceServer,
    /// Refers to an icon representing the context.
    ///
    /// Source: HTML.
    Icon,
    /// Refers to an index for the context.
    ///
    /// Source: HTML 4.01.
    Index,
    /// Refers to a time interval ending before the context interval.
    ///
    /// Source: Time Ontology in OWL, Section 4.2.21.
    IntervalAfter,
    /// Refers to a time interval starting after the context interval.
    ///
    /// Source: Time Ontology in OWL, Section 4.2.22.
    IntervalBefore,
    /// Refers to a time interval contained within the context interval.
    ///
    /// Source: Time Ontology in OWL, Section 4.2.23.
    IntervalContains,
    /// Refers to a time interval disjoint from the context interval.
    ///
    /// Source: Time Ontology in OWL, Section 4.2.24.
    IntervalDisjoint,
    /// Refers to a time interval containing the context interval.
    ///
    /// Source: Time Ontology in OWL, Section 4.2.25.
    IntervalDuring,
    /// Refers to a time interval matching the context interval.
    ///
    /// Source: Time Ontology in OWL, Section 4.2.26.
    IntervalEquals,
    /// Refers to a time interval with matching end to the context.
    ///
    /// Source: Time Ontology in OWL, Section 4.2.27.
    IntervalFinishedBy,
    /// Refers to a time interval finishing at the context end.
    ///
    /// Source: Time Ontology in OWL, Section 4.2.28.
    IntervalFinishes,
    /// Refers to a time interval encompassing the context interval.
    ///
    /// Source: Time Ontology in OWL, Section 4.2.29.
    IntervalIn,
    /// Refers to a time interval starting at the context end.
    ///
    /// Source: Time Ontology in OWL, Section 4.2.30.
    IntervalMeets,
    /// Refers to a time interval ending at the context start.
    ///
    /// Source: Time Ontology in OWL, Section 4.2.31.
    IntervalMetBy,
    /// Refers to a time interval overlapping from before context start.
    ///
    /// Source: Time Ontology in OWL, Section 4.2.32.
    IntervalOverlappedBy,
    /// Refers to a time interval overlapping past the context end.
    ///
    /// Source: Time Ontology in OWL, Section 4.2.33.
    IntervalOverlaps,
    /// Refers to a time interval with matching start to the context.
    ///
    /// Source: Time Ontology in OWL, Section 4.2.34.
    IntervalStartedBy,
    /// Refers to a time interval starting at the context start.
    ///
    /// Source: Time Ontology in OWL, Section 4.2.35.
    IntervalStarts,
    /// Refers to a member of a collection.
    ///
    /// Source: RFC 6573.
    Item,
    /// Refers to the furthest following resource in a series.
    ///
    /// Source: RFC 8288.
    Last,
    /// Points to a resource with the latest/current version.
    ///
    /// Source: RFC 5829.
    LatestVersion,
    /// Refers to a license associated with the context.
    ///
    /// Source: RFC 4946.
    License,
    /// Refers to a set of links with the context as a participant.
    ///
    /// Source: RFC 9264.
    Linkset,
    /// Refers to further information as a Link-based Resource Descriptor.
    ///
    /// Source: RFC 6415.
    Lrdd,
    /// Links to a manifest file for the context.
    ///
    /// Source: Web App Manifest.
    Manifest,
    /// Refers to a mask applicable to the icon.
    ///
    /// Source: Creating Pinned Tab Icons (Apple).
    MaskIcon,
    /// Refers to a resource about the author of the link's context.
    ///
    /// Source: Microformats rel=me.
    Me,
    /// Refers to a feed of personalized media recommendations.
    ///
    /// Source: Media Feeds.
    MediaFeed,
    /// Indicates a fixed resource representing a prior state.
    ///
    /// Source: RFC 7089.
    Memento,
    /// Refers to the context's Micropub endpoint.
    ///
    /// Source: Micropub.
    Micropub,
    /// Refers to a module for preemptive fetching.
    ///
    /// Source: HTML.
    ModulePreload,
    /// Refers to a resource for monitoring changes.
    ///
    /// Source: RFC 5989.
    Monitor,
    /// Refers to a resource for monitoring a group of resources.
    ///
    /// Source: RFC 5989.
    MonitorGroup,
    /// Refers to the next resource in a series.
    ///
    /// Source: HTML.
    Next,
    /// Refers to the immediately following archive resource.
    ///
    /// Source: RFC 5005.
    NextArchive,
    /// Indicates the author does not endorse the link target.
    ///
    /// Source: HTML.
    NoFollow,
    /// Indicates the new context should not be an auxiliary browsing context.
    ///
    /// Source: HTML.
    NoOpener,
    /// Indicates no referrer information should be leaked.
    ///
    /// Source: HTML.
    NoReferrer,
    /// Indicates the new context is an auxiliary browsing context.
    ///
    /// Source: HTML.
    Opener,
    /// Refers to a server for OpenID Authentication identity assertion.
    ///
    /// Source: OpenID Authentication 2.0.
    OpenId2LocalId,
    /// Refers to a server accepting OpenID Authentication protocol messages.
    ///
    /// Source: OpenID Authentication 2.0.
    OpenId2Provider,
    /// Refers to an Original Resource in a Memento context.
    ///
    /// Source: RFC 7089.
    Original,
    /// Refers to a P3P privacy policy.
    ///
    /// Source: The Platform for Privacy Preferences 1.0.
    P3pv1,
    /// Indicates a resource where payment is accepted.
    ///
    /// Source: RFC 8288.
    Payment,
    /// Refers to a Pingback resource address.
    ///
    /// Source: Pingback 1.0.
    Pingback,
    /// Indicates an origin to which early connection should be made.
    ///
    /// Source: HTML.
    Preconnect,
    /// Points to a resource with a predecessor version in history.
    ///
    /// Source: RFC 5829.
    PredecessorVersion,
    /// Indicates a resource that should be fetched early.
    ///
    /// Source: HTML.
    Prefetch,
    /// Indicates a resource that should be loaded early without blocking.
    ///
    /// Source: Preload.
    Preload,
    /// Indicates a resource to fetch and execute for next navigation.
    ///
    /// Source: Resource Hints.
    Prerender,
    /// Refers to the previous resource in a series.
    ///
    /// Source: HTML.
    Prev,
    /// Refers to a preview of the context.
    ///
    /// Source: RFC 6903, Section 3.
    Preview,
    /// Refers to the previous resource in a series (synonym for `prev`).
    ///
    /// Source: HTML 4.01.
    Previous,
    /// Refers to the immediately preceding archive resource.
    ///
    /// Source: RFC 5005.
    PrevArchive,
    /// Refers to a privacy policy for the context.
    ///
    /// Source: RFC 6903, Section 4.
    PrivacyPolicy,
    /// Identifies a resource conforming to a profile.
    ///
    /// Source: RFC 6906.
    Profile,
    /// Links to the publication manifest with metadata.
    ///
    /// Source: Publication Manifest.
    Publication,
    /// Used in RDAP RIR search to filter for "active" objects.
    ///
    /// Source: RFC 9910.
    RdapActive,
    /// Used in RDAP to refer to child objects without children.
    ///
    /// Source: RFC 9910.
    RdapBottom,
    /// Used in RDAP to refer to a set of child objects.
    ///
    /// Source: RFC 9910.
    RdapDown,
    /// Used in RDAP to refer to the topmost parent in a hierarchy.
    ///
    /// Source: RFC 9910.
    RdapTop,
    /// Used in RDAP to refer to a parent object in a hierarchy.
    ///
    /// Source: RFC 9910.
    RdapUp,
    /// Identifies a related resource.
    ///
    /// Source: RFC 4287.
    Related,
    /// Identifies a resource replying to the context.
    ///
    /// Source: RFC 4685.
    Replies,
    /// Identifies the root of a RESTCONF API.
    ///
    /// Source: RFC 8040.
    Restconf,
    /// Refers to an input value for a rule instance.
    ///
    /// Source: OCF Core Optional 2.2.0.
    RuleInput,
    /// Refers to a resource for searching the context.
    ///
    /// Source: OpenSearch.
    Search,
    /// Refers to a section in a collection of resources.
    ///
    /// Source: HTML 4.01.
    Section,
    /// Conveys an identifier for the link's context.
    ///
    /// Source: RFC 4287.
    ///
    /// Note: Named `Self_` to avoid conflict with the Rust keyword.
    Self_,
    /// Indicates a URI for a service document.
    ///
    /// Source: RFC 5023.
    Service,
    /// Identifies a service description for machines.
    ///
    /// Source: RFC 8631.
    ServiceDesc,
    /// Identifies a service documentation for humans.
    ///
    /// Source: RFC 8631.
    ServiceDoc,
    /// Identifies general metadata for machines.
    ///
    /// Source: RFC 8631.
    ServiceMeta,
    /// Refers to a SIP trunking capability set document.
    ///
    /// Source: RFC 9409.
    SipTrunkingCapability,
    /// Indicates a sponsored resource within the context.
    ///
    /// Source: Qualify Outbound Links (Google).
    Sponsored,
    /// Refers to the first resource in a collection.
    ///
    /// Source: HTML 4.01.
    Start,
    /// Identifies a resource representing the context's status.
    ///
    /// Source: RFC 8631.
    Status,
    /// Refers to a stylesheet.
    ///
    /// Source: HTML.
    Stylesheet,
    /// Refers to a subsection in a collection of resources.
    ///
    /// Source: HTML 4.01.
    Subsection,
    /// Points to a resource with a successor version in history.
    ///
    /// Source: RFC 5829.
    SuccessorVersion,
    /// Identifies a resource with retirement policy information.
    ///
    /// Source: RFC 8594.
    Sunset,
    /// Refers to a tag applying to the document.
    ///
    /// Source: HTML.
    Tag,
    /// Refers to terms of service for the context.
    ///
    /// Source: RFC 6903, Section 5.
    TermsOfService,
    /// Refers to a TimeGate for an Original Resource.
    ///
    /// Source: RFC 7089.
    TimeGate,
    /// Refers to a TimeMap for an Original Resource.
    ///
    /// Source: RFC 7089.
    TimeMap,
    /// Refers to a resource identifying the abstract semantic type.
    ///
    /// Source: RFC 6903, Section 6.
    Type,
    /// Indicates user-generated content within the context.
    ///
    /// Source: Qualify Outbound Links (Google).
    Ugc,
    /// Refers to a parent document in a hierarchy.
    ///
    /// Source: RFC 8288.
    Up,
    /// Points to a version history resource.
    ///
    /// Source: RFC 5829.
    VersionHistory,
    /// Identifies a source of information in the context.
    ///
    /// Source: RFC 4287.
    Via,
    /// Identifies a target supporting the Webmention protocol.
    ///
    /// Source: Webmention.
    Webmention,
    /// Points to a working copy for the resource.
    ///
    /// Source: RFC 5829.
    WorkingCopy,
    /// Points to the versioned resource source of a working copy.
    ///
    /// Source: RFC 5829.
    WorkingCopyOf,
}

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
