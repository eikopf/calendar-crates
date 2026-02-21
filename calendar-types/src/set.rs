//! Types for finite set values.

use std::{convert::Infallible, fmt, str::FromStr};

use strum::{Display, EnumString};

/// A token which may be a statically known value of type `T` or else an unknown value of type
/// `S`.
///
/// The principal use of this type is to allow finite enums to be extended with arbitrary values,
/// most commonly some unknown string which is permissible but statically unknowable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Token<T, S> {
    /// A statically known value.
    Known(T),
    /// An unknown or vendor-defined value.
    Unknown(S),
}

impl<T: Default, S> Default for Token<T, S> {
    fn default() -> Self {
        Self::Known(Default::default())
    }
}

impl<T, S> FromStr for Token<T, S>
where
    T: FromStr,
    for<'a> &'a str: Into<S>,
{
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match T::from_str(s) {
            Ok(value) => Ok(Token::Known(value)),
            Err(_) => Ok(Token::Unknown(s.into())),
        }
    }
}

impl<T, S> Token<T, S> {
    /// Like [`FromStr`], but uses a fallible conversion for the unknown variant.
    pub fn try_from_str<'a>(s: &'a str) -> Result<Self, <&'a str as TryInto<S>>::Error>
    where
        T: FromStr,
        &'a str: TryInto<S>,
    {
        match T::from_str(s) {
            Ok(value) => Ok(Token::Known(value)),
            Err(_) => s.try_into().map(Token::Unknown),
        }
    }

    /// Maps the unknown value of a `Token`, leaving known values unchanged.
    pub fn map_unknown<U>(self, f: impl FnOnce(S) -> U) -> Token<T, U> {
        match self {
            Token::Known(t) => Token::Known(t),
            Token::Unknown(s) => Token::Unknown(f(s)),
        }
    }
}

impl<T: fmt::Display, S: fmt::Display> fmt::Display for Token<T, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Known(t) => fmt::Display::fmt(t, f),
            Token::Unknown(s) => fmt::Display::fmt(s, f),
        }
    }
}

/// A link relation from the [IANA Link Relations Registry].
///
/// [IANA Link Relations Registry]: https://www.iana.org/assignments/link-relations/
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumString, Display)]
#[non_exhaustive]
#[strum(ascii_case_insensitive)]
pub enum LinkRelation {
    /// Refers to a resource that is the subject of the link's context.
    ///
    /// Source: RFC 6903, Section 2.
    #[strum(serialize = "about")]
    About,
    /// Asserts that the link target provides an access control description.
    ///
    /// Source: Web Access Control.
    #[strum(serialize = "acl")]
    Acl,
    /// Refers to a substitute for this context.
    ///
    /// Source: HTML.
    #[strum(serialize = "alternate")]
    Alternate,
    /// Refers to an AMP HTML version of the context.
    ///
    /// Source: AMP HTML.
    #[strum(serialize = "amphtml")]
    AmpHtml,
    /// Refers to a list of published APIs.
    ///
    /// Source: RFC 9727.
    #[strum(serialize = "api-catalog")]
    ApiCatalog,
    /// Refers to an appendix in a collection of resources.
    ///
    /// Source: HTML 4.01.
    #[strum(serialize = "appendix")]
    Appendix,
    /// Refers to an icon for the context (synonym for `icon`).
    ///
    /// Source: Apple Web Application Configuration.
    #[strum(serialize = "apple-touch-icon")]
    AppleTouchIcon,
    /// Refers to a launch screen for the context.
    ///
    /// Source: Apple Web Application Configuration.
    #[strum(serialize = "apple-touch-startup-image")]
    AppleTouchStartupImage,
    /// Refers to a collection of records, documents, or other materials.
    ///
    /// Source: HTML 5.
    #[strum(serialize = "archives")]
    Archives,
    /// Refers to the context's author.
    ///
    /// Source: HTML.
    #[strum(serialize = "author")]
    Author,
    /// Identifies the entity that blocks access to a resource.
    ///
    /// Source: RFC 7725.
    #[strum(serialize = "blocked-by")]
    BlockedBy,
    /// Gives a permanent link to use for bookmarking purposes.
    ///
    /// Source: HTML.
    #[strum(serialize = "bookmark")]
    Bookmark,
    /// Links to a C2PA Manifest associated with the link context.
    ///
    /// Source: C2PA Technical Specification.
    #[strum(serialize = "c2pa-manifest")]
    C2paManifest,
    /// Designates the preferred version of a resource.
    ///
    /// Source: RFC 6596.
    #[strum(serialize = "canonical")]
    Canonical,
    /// Refers to a chapter in a collection of resources.
    ///
    /// Source: HTML 4.01.
    #[strum(serialize = "chapter")]
    Chapter,
    /// Indicates the preferred URI for permanent citation.
    ///
    /// Source: RFC 8574.
    #[strum(serialize = "cite-as")]
    CiteAs,
    /// Represents a collection resource for the context.
    ///
    /// Source: RFC 6573.
    #[strum(serialize = "collection")]
    Collection,
    /// Refers to a compression dictionary for content encoding.
    ///
    /// Source: RFC 9842, Section 3.
    #[strum(serialize = "compression-dictionary")]
    CompressionDictionary,
    /// Refers to a table of contents.
    ///
    /// Source: HTML 4.01.
    #[strum(serialize = "contents")]
    Contents,
    /// The document was later converted from the linked document.
    ///
    /// Source: RFC 7991.
    #[strum(serialize = "convertedfrom")]
    ConvertedFrom,
    /// Refers to a copyright statement for the context.
    ///
    /// Source: HTML 4.01.
    #[strum(serialize = "copyright")]
    Copyright,
    /// Refers to a resource from which a submission form can be obtained.
    ///
    /// Source: RFC 6861.
    #[strum(serialize = "create-form")]
    CreateForm,
    /// Refers to the most recent item(s) in a collection.
    ///
    /// Source: RFC 5005.
    #[strum(serialize = "current")]
    Current,
    /// Links to documentation about deprecation of the context.
    ///
    /// Source: RFC 9745, Section 3.
    #[strum(serialize = "deprecation")]
    Deprecation,
    /// Refers to a resource providing information about the context.
    ///
    /// Source: Protocol for Web Description Resources (POWDER).
    #[strum(serialize = "describedby")]
    DescribedBy,
    /// The relationship asserts that resource A describes resource B.
    ///
    /// Source: RFC 6892.
    #[strum(serialize = "describes")]
    Describes,
    /// Refers to a list of patent disclosures.
    ///
    /// Source: RFC 6579.
    #[strum(serialize = "disclosure")]
    Disclosure,
    /// Indicates an origin whose resources should be resolved early.
    ///
    /// Source: HTML.
    #[strum(serialize = "dns-prefetch")]
    DnsPrefetch,
    /// Refers to a resource with byte-for-byte identical representation.
    ///
    /// Source: RFC 6249.
    #[strum(serialize = "duplicate")]
    Duplicate,
    /// Refers to a resource that can be used to edit the context.
    ///
    /// Source: RFC 5023.
    #[strum(serialize = "edit")]
    Edit,
    /// Refers to a submission form for editing the context.
    ///
    /// Source: RFC 6861.
    #[strum(serialize = "edit-form")]
    EditForm,
    /// Refers to a resource for editing media associated with the context.
    ///
    /// Source: RFC 5023.
    #[strum(serialize = "edit-media")]
    EditMedia,
    /// Identifies a potentially large related resource.
    ///
    /// Source: RFC 4287.
    #[strum(serialize = "enclosure")]
    Enclosure,
    /// Indicates a resource not part of the same site as the context.
    ///
    /// Source: HTML.
    #[strum(serialize = "external")]
    External,
    /// Refers to the furthest preceding resource in a series.
    ///
    /// Source: RFC 8288.
    #[strum(serialize = "first")]
    First,
    /// Refers to a resource providing IP geolocation information.
    ///
    /// Source: RFC 9877.
    #[strum(serialize = "geofeed")]
    Geofeed,
    /// Refers to a glossary of terms.
    ///
    /// Source: HTML 4.01.
    #[strum(serialize = "glossary")]
    Glossary,
    /// Refers to context-sensitive help.
    ///
    /// Source: HTML.
    #[strum(serialize = "help")]
    Help,
    /// Refers to a resource hosted by the server.
    ///
    /// Source: RFC 6690.
    #[strum(serialize = "hosts")]
    Hosts,
    /// Refers to a hub for subscribing to updates.
    ///
    /// Source: WebSub.
    #[strum(serialize = "hub")]
    Hub,
    /// Refers to STUN/TURN server information for ICE connections.
    ///
    /// Source: RFC 9725.
    #[strum(serialize = "ice-server")]
    IceServer,
    /// Refers to an icon representing the context.
    ///
    /// Source: HTML.
    #[strum(serialize = "icon")]
    Icon,
    /// Refers to an index for the context.
    ///
    /// Source: HTML 4.01.
    #[strum(serialize = "index")]
    Index,
    /// Refers to a time interval ending before the context interval.
    ///
    /// Source: Time Ontology in OWL, Section 4.2.21.
    #[strum(serialize = "intervalafter")]
    IntervalAfter,
    /// Refers to a time interval starting after the context interval.
    ///
    /// Source: Time Ontology in OWL, Section 4.2.22.
    #[strum(serialize = "intervalbefore")]
    IntervalBefore,
    /// Refers to a time interval contained within the context interval.
    ///
    /// Source: Time Ontology in OWL, Section 4.2.23.
    #[strum(serialize = "intervalcontains")]
    IntervalContains,
    /// Refers to a time interval disjoint from the context interval.
    ///
    /// Source: Time Ontology in OWL, Section 4.2.24.
    #[strum(serialize = "intervaldisjoint")]
    IntervalDisjoint,
    /// Refers to a time interval containing the context interval.
    ///
    /// Source: Time Ontology in OWL, Section 4.2.25.
    #[strum(serialize = "intervalduring")]
    IntervalDuring,
    /// Refers to a time interval matching the context interval.
    ///
    /// Source: Time Ontology in OWL, Section 4.2.26.
    #[strum(serialize = "intervalequals")]
    IntervalEquals,
    /// Refers to a time interval with matching end to the context.
    ///
    /// Source: Time Ontology in OWL, Section 4.2.27.
    #[strum(serialize = "intervalfinishedby")]
    IntervalFinishedBy,
    /// Refers to a time interval finishing at the context end.
    ///
    /// Source: Time Ontology in OWL, Section 4.2.28.
    #[strum(serialize = "intervalfinishes")]
    IntervalFinishes,
    /// Refers to a time interval encompassing the context interval.
    ///
    /// Source: Time Ontology in OWL, Section 4.2.29.
    #[strum(serialize = "intervalin")]
    IntervalIn,
    /// Refers to a time interval starting at the context end.
    ///
    /// Source: Time Ontology in OWL, Section 4.2.30.
    #[strum(serialize = "intervalmeets")]
    IntervalMeets,
    /// Refers to a time interval ending at the context start.
    ///
    /// Source: Time Ontology in OWL, Section 4.2.31.
    #[strum(serialize = "intervalmetby")]
    IntervalMetBy,
    /// Refers to a time interval overlapping from before context start.
    ///
    /// Source: Time Ontology in OWL, Section 4.2.32.
    #[strum(serialize = "intervaloverlappedby")]
    IntervalOverlappedBy,
    /// Refers to a time interval overlapping past the context end.
    ///
    /// Source: Time Ontology in OWL, Section 4.2.33.
    #[strum(serialize = "intervaloverlaps")]
    IntervalOverlaps,
    /// Refers to a time interval with matching start to the context.
    ///
    /// Source: Time Ontology in OWL, Section 4.2.34.
    #[strum(serialize = "intervalstartedby")]
    IntervalStartedBy,
    /// Refers to a time interval starting at the context start.
    ///
    /// Source: Time Ontology in OWL, Section 4.2.35.
    #[strum(serialize = "intervalstarts")]
    IntervalStarts,
    /// Refers to a member of a collection.
    ///
    /// Source: RFC 6573.
    #[strum(serialize = "item")]
    Item,
    /// Refers to the furthest following resource in a series.
    ///
    /// Source: RFC 8288.
    #[strum(serialize = "last")]
    Last,
    /// Points to a resource with the latest/current version.
    ///
    /// Source: RFC 5829.
    #[strum(serialize = "latest-version")]
    LatestVersion,
    /// Refers to a license associated with the context.
    ///
    /// Source: RFC 4946.
    #[strum(serialize = "license")]
    License,
    /// Refers to a set of links with the context as a participant.
    ///
    /// Source: RFC 9264.
    #[strum(serialize = "linkset")]
    Linkset,
    /// Refers to further information as a Link-based Resource Descriptor.
    ///
    /// Source: RFC 6415.
    #[strum(serialize = "lrdd")]
    Lrdd,
    /// Links to a manifest file for the context.
    ///
    /// Source: Web App Manifest.
    #[strum(serialize = "manifest")]
    Manifest,
    /// Refers to a mask applicable to the icon.
    ///
    /// Source: Creating Pinned Tab Icons (Apple).
    #[strum(serialize = "mask-icon")]
    MaskIcon,
    /// Refers to a resource about the author of the link's context.
    ///
    /// Source: Microformats rel=me.
    #[strum(serialize = "me")]
    Me,
    /// Refers to a feed of personalized media recommendations.
    ///
    /// Source: Media Feeds.
    #[strum(serialize = "media-feed")]
    MediaFeed,
    /// Indicates a fixed resource representing a prior state.
    ///
    /// Source: RFC 7089.
    #[strum(serialize = "memento")]
    Memento,
    /// Refers to the context's Micropub endpoint.
    ///
    /// Source: Micropub.
    #[strum(serialize = "micropub")]
    Micropub,
    /// Refers to a module for preemptive fetching.
    ///
    /// Source: HTML.
    #[strum(serialize = "modulepreload")]
    ModulePreload,
    /// Refers to a resource for monitoring changes.
    ///
    /// Source: RFC 5989.
    #[strum(serialize = "monitor")]
    Monitor,
    /// Refers to a resource for monitoring a group of resources.
    ///
    /// Source: RFC 5989.
    #[strum(serialize = "monitor-group")]
    MonitorGroup,
    /// Refers to the next resource in a series.
    ///
    /// Source: HTML.
    #[strum(serialize = "next")]
    Next,
    /// Refers to the immediately following archive resource.
    ///
    /// Source: RFC 5005.
    #[strum(serialize = "next-archive")]
    NextArchive,
    /// Indicates the author does not endorse the link target.
    ///
    /// Source: HTML.
    #[strum(serialize = "nofollow")]
    NoFollow,
    /// Indicates the new context should not be an auxiliary browsing context.
    ///
    /// Source: HTML.
    #[strum(serialize = "noopener")]
    NoOpener,
    /// Indicates no referrer information should be leaked.
    ///
    /// Source: HTML.
    #[strum(serialize = "noreferrer")]
    NoReferrer,
    /// Indicates the new context is an auxiliary browsing context.
    ///
    /// Source: HTML.
    #[strum(serialize = "opener")]
    Opener,
    /// Refers to a server for OpenID Authentication identity assertion.
    ///
    /// Source: OpenID Authentication 2.0.
    #[strum(serialize = "openid2.local_id")]
    OpenId2LocalId,
    /// Refers to a server accepting OpenID Authentication protocol messages.
    ///
    /// Source: OpenID Authentication 2.0.
    #[strum(serialize = "openid2.provider")]
    OpenId2Provider,
    /// Refers to an Original Resource in a Memento context.
    ///
    /// Source: RFC 7089.
    #[strum(serialize = "original")]
    Original,
    /// Refers to a P3P privacy policy.
    ///
    /// Source: The Platform for Privacy Preferences 1.0.
    #[strum(serialize = "p3pv1")]
    P3pv1,
    /// Indicates a resource where payment is accepted.
    ///
    /// Source: RFC 8288.
    #[strum(serialize = "payment")]
    Payment,
    /// Refers to a Pingback resource address.
    ///
    /// Source: Pingback 1.0.
    #[strum(serialize = "pingback")]
    Pingback,
    /// Indicates an origin to which early connection should be made.
    ///
    /// Source: HTML.
    #[strum(serialize = "preconnect")]
    Preconnect,
    /// Points to a resource with a predecessor version in history.
    ///
    /// Source: RFC 5829.
    #[strum(serialize = "predecessor-version")]
    PredecessorVersion,
    /// Indicates a resource that should be fetched early.
    ///
    /// Source: HTML.
    #[strum(serialize = "prefetch")]
    Prefetch,
    /// Indicates a resource that should be loaded early without blocking.
    ///
    /// Source: Preload.
    #[strum(serialize = "preload")]
    Preload,
    /// Indicates a resource to fetch and execute for next navigation.
    ///
    /// Source: Resource Hints.
    #[strum(serialize = "prerender")]
    Prerender,
    /// Refers to the previous resource in a series.
    ///
    /// Source: HTML.
    #[strum(serialize = "prev")]
    Prev,
    /// Refers to a preview of the context.
    ///
    /// Source: RFC 6903, Section 3.
    #[strum(serialize = "preview")]
    Preview,
    /// Refers to the previous resource in a series (synonym for `prev`).
    ///
    /// Source: HTML 4.01.
    #[strum(serialize = "previous")]
    Previous,
    /// Refers to the immediately preceding archive resource.
    ///
    /// Source: RFC 5005.
    #[strum(serialize = "prev-archive")]
    PrevArchive,
    /// Refers to a privacy policy for the context.
    ///
    /// Source: RFC 6903, Section 4.
    #[strum(serialize = "privacy-policy")]
    PrivacyPolicy,
    /// Identifies a resource conforming to a profile.
    ///
    /// Source: RFC 6906.
    #[strum(serialize = "profile")]
    Profile,
    /// Links to the publication manifest with metadata.
    ///
    /// Source: Publication Manifest.
    #[strum(serialize = "publication")]
    Publication,
    /// Used in RDAP RIR search to filter for "active" objects.
    ///
    /// Source: RFC 9910.
    #[strum(serialize = "rdap-active")]
    RdapActive,
    /// Used in RDAP to refer to child objects without children.
    ///
    /// Source: RFC 9910.
    #[strum(serialize = "rdap-bottom")]
    RdapBottom,
    /// Used in RDAP to refer to a set of child objects.
    ///
    /// Source: RFC 9910.
    #[strum(serialize = "rdap-down")]
    RdapDown,
    /// Used in RDAP to refer to the topmost parent in a hierarchy.
    ///
    /// Source: RFC 9910.
    #[strum(serialize = "rdap-top")]
    RdapTop,
    /// Used in RDAP to refer to a parent object in a hierarchy.
    ///
    /// Source: RFC 9910.
    #[strum(serialize = "rdap-up")]
    RdapUp,
    /// Identifies a related resource.
    ///
    /// Source: RFC 4287.
    #[strum(serialize = "related")]
    Related,
    /// Identifies a resource replying to the context.
    ///
    /// Source: RFC 4685.
    #[strum(serialize = "replies")]
    Replies,
    /// Identifies the root of a RESTCONF API.
    ///
    /// Source: RFC 8040.
    #[strum(serialize = "restconf")]
    Restconf,
    /// Refers to an input value for a rule instance.
    ///
    /// Source: OCF Core Optional 2.2.0.
    #[strum(serialize = "ruleinput")]
    RuleInput,
    /// Refers to a resource for searching the context.
    ///
    /// Source: OpenSearch.
    #[strum(serialize = "search")]
    Search,
    /// Refers to a section in a collection of resources.
    ///
    /// Source: HTML 4.01.
    #[strum(serialize = "section")]
    Section,
    /// Conveys an identifier for the link's context.
    ///
    /// Source: RFC 4287.
    ///
    /// Note: Named `Self_` to avoid conflict with the Rust keyword.
    #[strum(serialize = "self")]
    Self_,
    /// Indicates a URI for a service document.
    ///
    /// Source: RFC 5023.
    #[strum(serialize = "service")]
    Service,
    /// Identifies a service description for machines.
    ///
    /// Source: RFC 8631.
    #[strum(serialize = "service-desc")]
    ServiceDesc,
    /// Identifies a service documentation for humans.
    ///
    /// Source: RFC 8631.
    #[strum(serialize = "service-doc")]
    ServiceDoc,
    /// Identifies general metadata for machines.
    ///
    /// Source: RFC 8631.
    #[strum(serialize = "service-meta")]
    ServiceMeta,
    /// Refers to a SIP trunking capability set document.
    ///
    /// Source: RFC 9409.
    #[strum(serialize = "sip-trunking-capability")]
    SipTrunkingCapability,
    /// Indicates a sponsored resource within the context.
    ///
    /// Source: Qualify Outbound Links (Google).
    #[strum(serialize = "sponsored")]
    Sponsored,
    /// Refers to the first resource in a collection.
    ///
    /// Source: HTML 4.01.
    #[strum(serialize = "start")]
    Start,
    /// Identifies a resource representing the context's status.
    ///
    /// Source: RFC 8631.
    #[strum(serialize = "status")]
    Status,
    /// Refers to a stylesheet.
    ///
    /// Source: HTML.
    #[strum(serialize = "stylesheet")]
    Stylesheet,
    /// Refers to a subsection in a collection of resources.
    ///
    /// Source: HTML 4.01.
    #[strum(serialize = "subsection")]
    Subsection,
    /// Points to a resource with a successor version in history.
    ///
    /// Source: RFC 5829.
    #[strum(serialize = "successor-version")]
    SuccessorVersion,
    /// Identifies a resource with retirement policy information.
    ///
    /// Source: RFC 8594.
    #[strum(serialize = "sunset")]
    Sunset,
    /// Refers to a tag applying to the document.
    ///
    /// Source: HTML.
    #[strum(serialize = "tag")]
    Tag,
    /// Refers to terms of service for the context.
    ///
    /// Source: RFC 6903, Section 5.
    #[strum(serialize = "terms-of-service")]
    TermsOfService,
    /// Refers to a TimeGate for an Original Resource.
    ///
    /// Source: RFC 7089.
    #[strum(serialize = "timegate")]
    TimeGate,
    /// Refers to a TimeMap for an Original Resource.
    ///
    /// Source: RFC 7089.
    #[strum(serialize = "timemap")]
    TimeMap,
    /// Refers to a resource identifying the abstract semantic type.
    ///
    /// Source: RFC 6903, Section 6.
    #[strum(serialize = "type")]
    Type,
    /// Indicates user-generated content within the context.
    ///
    /// Source: Qualify Outbound Links (Google).
    #[strum(serialize = "ugc")]
    Ugc,
    /// Refers to a parent document in a hierarchy.
    ///
    /// Source: RFC 8288.
    #[strum(serialize = "up")]
    Up,
    /// Points to a version history resource.
    ///
    /// Source: RFC 5829.
    #[strum(serialize = "version-history")]
    VersionHistory,
    /// Identifies a source of information in the context.
    ///
    /// Source: RFC 4287.
    #[strum(serialize = "via")]
    Via,
    /// Identifies a target supporting the Webmention protocol.
    ///
    /// Source: Webmention.
    #[strum(serialize = "webmention")]
    Webmention,
    /// Points to a working copy for the resource.
    ///
    /// Source: RFC 5829.
    #[strum(serialize = "working-copy")]
    WorkingCopy,
    /// Points to the versioned resource source of a working copy.
    ///
    /// Source: RFC 5829.
    #[strum(serialize = "working-copy-of")]
    WorkingCopyOf,
}

/// A location type from the [IANA Location Types Registry].
///
/// [IANA Location Types Registry]: https://www.iana.org/assignments/location-type-registry/
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumString, Display)]
#[non_exhaustive]
#[strum(ascii_case_insensitive)]
pub enum LocationType {
    /// A device used for flight (airplane, helicopter, glider, balloon, etc.).
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "aircraft")]
    Aircraft,
    /// A place from which aircraft operate (airport, heliport).
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "airport")]
    Airport,
    /// Enclosed area used for sports events.
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "arena")]
    Arena,
    /// An automotive vehicle designed for passenger transportation.
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "automobile")]
    Automobile,
    /// Business establishment for financial services.
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "bank")]
    Bank,
    /// A bar or saloon.
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "bar")]
    Bar,
    /// A two-wheeled pedal-propelled vehicle.
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "bicycle")]
    Bicycle,
    /// A large motor vehicle designed to carry passengers.
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "bus")]
    Bus,
    /// Terminal that serves bus passengers (bus depot, bus terminal).
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "bus-station")]
    BusStation,
    /// A small informal establishment serving refreshments; coffee shop.
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "cafe")]
    Cafe,
    /// An area used for camping, often with facilities for tents or RVs.
    ///
    /// Source: NH Division of Emergency Services and Communications.
    #[strum(serialize = "campground")]
    Campground,
    /// A facility providing care services (nursing home, assisted living).
    ///
    /// Source: NH Division of Emergency Services and Communications.
    #[strum(serialize = "care-facility")]
    CareFacility,
    /// Academic classroom or lecture hall.
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "classroom")]
    Classroom,
    /// Dance club, nightclub, or discotheque.
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "club")]
    Club,
    /// Construction site.
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "construction")]
    Construction,
    /// Convention center or exhibition hall.
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "convention-center")]
    ConventionCenter,
    /// A building or housing unit that is not attached to other buildings.
    ///
    /// Source: NH Division of Emergency Services and Communications.
    #[strum(serialize = "detached-unit")]
    DetachedUnit,
    /// A facility housing firefighting equipment and personnel.
    ///
    /// Source: NH Division of Emergency Services and Communications.
    #[strum(serialize = "fire-station")]
    FireStation,
    /// Government building (legislative, executive, judicial, police, military).
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "government")]
    Government,
    /// Hospital, hospice, medical clinic, mental institution, or doctor's office.
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "hospital")]
    Hospital,
    /// Hotel, motel, inn, or other lodging establishment.
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "hotel")]
    Hotel,
    /// Industrial setting (manufacturing floor, power plant).
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "industrial")]
    Industrial,
    /// A location identified by a landmark or notable address.
    ///
    /// Source: NH Division of Emergency Services and Communications.
    #[strum(serialize = "landmark-address")]
    LandmarkAddress,
    /// Library or other public place for literary and artistic materials.
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "library")]
    Library,
    /// A two-wheeled automotive vehicle, including a scooter.
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "motorcycle")]
    Motorcycle,
    /// A garage owned or operated by a municipality.
    ///
    /// Source: NH Division of Emergency Services and Communications.
    #[strum(serialize = "municipal-garage")]
    MunicipalGarage,
    /// A building housing collections of artifacts or specimens.
    ///
    /// Source: NENA-STA-004.1.1-2025, NG9-1-1 CLDXF-US.
    #[strum(serialize = "museum")]
    Museum,
    /// Business setting, such as an office.
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "office")]
    Office,
    /// A place without a registered place type representation.
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "other")]
    Other,
    /// Outside a building, in the open air (park, city streets).
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "outdoors")]
    Outdoors,
    /// A parking lot or parking garage.
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "parking")]
    Parking,
    /// A public telephone booth or kiosk.
    ///
    /// Source: NH Division of Emergency Services and Communications.
    #[strum(serialize = "phone-box")]
    PhoneBox,
    /// A religious site (church, chapel, mosque, shrine, synagogue, temple).
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "place-of-worship")]
    PlaceOfWorship,
    /// A post office or mail facility.
    ///
    /// Source: NH Division of Emergency Services and Communications.
    #[strum(serialize = "post-office")]
    PostOffice,
    /// Correctional institution (prison, penitentiary, jail, brig).
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "prison")]
    Prison,
    /// Public area (shopping mall, street, park, public building, conveyance).
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "public")]
    Public,
    /// Any form of public transport (aircraft, bus, train, ship).
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "public-transport")]
    PublicTransport,
    /// A private or residential setting.
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "residence")]
    Residence,
    /// Restaurant, coffee shop, or other public dining establishment.
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "restaurant")]
    Restaurant,
    /// School or university property.
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "school")]
    School,
    /// Shopping mall or area with stores accessible by common passageways.
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "shopping-area")]
    ShoppingArea,
    /// Large structure for sports events, including a racetrack.
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "stadium")]
    Stadium,
    /// Place where merchandise is offered for sale; a shop.
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "store")]
    Store,
    /// A public thoroughfare (avenue, street, alley, lane, road, sidewalk).
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "street")]
    Street,
    /// Theater, auditorium, movie theater, or similar presentation facility.
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "theater")]
    Theater,
    /// A booth or station for collecting tolls.
    ///
    /// Source: NH Division of Emergency Services and Communications.
    #[strum(serialize = "toll-booth")]
    TollBooth,
    /// A building housing local government offices.
    ///
    /// Source: NH Division of Emergency Services and Communications.
    #[strum(serialize = "town-hall")]
    TownHall,
    /// Train, monorail, maglev, cable car, or similar conveyance.
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "train")]
    Train,
    /// Terminal where trains load or unload passengers or goods.
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "train-station")]
    TrainStation,
    /// An automotive vehicle for hauling goods rather than people.
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "truck")]
    Truck,
    /// In a land-, water-, or aircraft that is in motion.
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "underway")]
    Underway,
    /// The type of place is unknown.
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "unknown")]
    Unknown,
    /// A utility box or cabinet housing utility equipment.
    ///
    /// Source: NH Division of Emergency Services and Communications.
    #[strum(serialize = "utilitybox")]
    UtilityBox,
    /// Place for storing goods (storehouse, self-storage facility).
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "warehouse")]
    Warehouse,
    /// A facility for processing or transferring waste materials.
    ///
    /// Source: NH Division of Emergency Services and Communications.
    #[strum(serialize = "waste-transfer-facility")]
    WasteTransferFacility,
    /// In, on, or above bodies of water (ocean, lake, river, canal).
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "water")]
    Water,
    /// A facility for water treatment or distribution.
    ///
    /// Source: NH Division of Emergency Services and Communications.
    #[strum(serialize = "water-facility")]
    WaterFacility,
    /// A vessel for travel on water (boat, ship).
    ///
    /// Source: RFC 4589.
    #[strum(serialize = "watercraft")]
    Watercraft,
    /// A camp providing programs and activities for youth.
    ///
    /// Source: NH Division of Emergency Services and Communications.
    #[strum(serialize = "youth-camp")]
    YouthCamp,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn link_relation_from_str_lowercase() {
        assert_eq!(
            LinkRelation::from_str("about").unwrap(),
            LinkRelation::About
        );
        assert_eq!(
            LinkRelation::from_str("api-catalog").unwrap(),
            LinkRelation::ApiCatalog
        );
        assert_eq!(LinkRelation::from_str("self").unwrap(), LinkRelation::Self_);
        assert_eq!(
            LinkRelation::from_str("openid2.provider").unwrap(),
            LinkRelation::OpenId2Provider
        );
    }

    #[test]
    fn link_relation_from_str_case_insensitive() {
        assert_eq!(
            LinkRelation::from_str("ABOUT").unwrap(),
            LinkRelation::About
        );
        assert_eq!(
            LinkRelation::from_str("About").unwrap(),
            LinkRelation::About
        );
        assert_eq!(
            LinkRelation::from_str("API-CATALOG").unwrap(),
            LinkRelation::ApiCatalog
        );
        assert_eq!(
            LinkRelation::from_str("Api-Catalog").unwrap(),
            LinkRelation::ApiCatalog
        );
        assert_eq!(LinkRelation::from_str("SELF").unwrap(), LinkRelation::Self_);
        assert_eq!(
            LinkRelation::from_str("OPENID2.PROVIDER").unwrap(),
            LinkRelation::OpenId2Provider
        );
    }

    #[test]
    fn link_relation_from_str_rejects_invalid() {
        assert!(LinkRelation::from_str("").is_err());
        assert!(LinkRelation::from_str("not-a-relation").is_err());
        assert!(LinkRelation::from_str("foobar").is_err());
        assert!(LinkRelation::from_str("about ").is_err());
        assert!(LinkRelation::from_str(" about").is_err());
    }

    #[test]
    fn link_relation_from_str_rejects_wrong_format() {
        // Snake case is NOT a valid alternative to kebab case
        assert!(LinkRelation::from_str("api_catalog").is_err());
        assert!(LinkRelation::from_str("cite_as").is_err());
        assert!(LinkRelation::from_str("edit_form").is_err());

        // PascalCase variant names are NOT valid
        assert!(LinkRelation::from_str("ApiCatalog").is_err());
        assert!(LinkRelation::from_str("CiteAs").is_err());
        assert!(LinkRelation::from_str("EditForm").is_err());

        // camelCase is NOT valid
        assert!(LinkRelation::from_str("apiCatalog").is_err());
        assert!(LinkRelation::from_str("citeAs").is_err());
    }

    #[test]
    fn location_type_from_str_lowercase() {
        assert_eq!(
            LocationType::from_str("aircraft").unwrap(),
            LocationType::Aircraft
        );
        assert_eq!(
            LocationType::from_str("bus-station").unwrap(),
            LocationType::BusStation
        );
        assert_eq!(
            LocationType::from_str("place-of-worship").unwrap(),
            LocationType::PlaceOfWorship
        );
        // Note: "utilitybox" has no hyphen per IANA registry
        assert_eq!(
            LocationType::from_str("utilitybox").unwrap(),
            LocationType::UtilityBox
        );
    }

    #[test]
    fn location_type_from_str_case_insensitive() {
        assert_eq!(
            LocationType::from_str("AIRCRAFT").unwrap(),
            LocationType::Aircraft
        );
        assert_eq!(
            LocationType::from_str("Aircraft").unwrap(),
            LocationType::Aircraft
        );
        assert_eq!(
            LocationType::from_str("BUS-STATION").unwrap(),
            LocationType::BusStation
        );
        assert_eq!(
            LocationType::from_str("Bus-Station").unwrap(),
            LocationType::BusStation
        );
        assert_eq!(
            LocationType::from_str("PLACE-OF-WORSHIP").unwrap(),
            LocationType::PlaceOfWorship
        );
        assert_eq!(
            LocationType::from_str("UTILITYBOX").unwrap(),
            LocationType::UtilityBox
        );
    }

    #[test]
    fn location_type_from_str_rejects_invalid() {
        assert!(LocationType::from_str("").is_err());
        assert!(LocationType::from_str("not-a-location").is_err());
        assert!(LocationType::from_str("foobar").is_err());
        assert!(LocationType::from_str("airport ").is_err());
        assert!(LocationType::from_str(" airport").is_err());
    }

    #[test]
    fn location_type_from_str_rejects_wrong_format() {
        // Snake case is NOT a valid alternative to kebab case
        assert!(LocationType::from_str("bus_station").is_err());
        assert!(LocationType::from_str("fire_station").is_err());
        assert!(LocationType::from_str("place_of_worship").is_err());

        // PascalCase variant names are NOT valid
        assert!(LocationType::from_str("BusStation").is_err());
        assert!(LocationType::from_str("FireStation").is_err());
        assert!(LocationType::from_str("PlaceOfWorship").is_err());

        // camelCase is NOT valid
        assert!(LocationType::from_str("busStation").is_err());
        assert!(LocationType::from_str("fireStation").is_err());

        // "utility-box" with hyphen is NOT valid (registry uses "utilitybox")
        assert!(LocationType::from_str("utility-box").is_err());
    }
}
