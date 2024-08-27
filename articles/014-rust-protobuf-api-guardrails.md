# Implementing Guardrails around `up-rust` UUri Protobuf Serialization

In uProtocol we use [`UUri`](https://github.com/eclipse-uprotocol/up-spec/blob/main/basics/uri.adoc)s, [encoded](https://github.com/eclipse-uprotocol/up-spec/blob/da5ca97d3a7541d2fcd52ed010bc3bcca92e46cb/up-core-api/uprotocol/v1/uri.proto#L25) in Protobuf .proto files to describe the source and sink for messages.

In an earlier version of the `UUri` spec and corresponding .proto file we defined its configuration in a way we thought at the time to be flexible enough for using both for general purpose use in high compute, cloud, and mobile and when communicating to mechatronics devices (think e.g. brake controller).

We later pivoted to a simpler representation of the `UUri` as we learned that flexibility was not going to be critical and took a lot more code to get everything done correctly.

I'll walk through the guardrails I implemented for an earlier version of `UUri` serialization validation checks before we would allow serialization down to a Protobuf object.

## `UUri` usage in uProtocol

Here's a quick tour of where `UUri` fits in.

[`UMessage`](https://github.com/eclipse-uprotocol/up-spec/blob/da5ca97d3a7541d2fcd52ed010bc3bcca92e46cb/up-core-api/uprotocol/v1/umessage.proto#L24-L33):

```
// UMessage is the top-level message type for the uProtocol.
// It contains a header (UAttributes), and payload and is a way of representing a
// message that would be sent between two uEs.
message UMessage {
    // uProtocol mandatory and optional attributes
    UAttributes attributes = 1;


    // Optional message payload
    optional bytes payload = 2;
}
```

[`UAttributes`](https://github.com/eclipse-uprotocol/up-spec/blob/da5ca97d3a7541d2fcd52ed010bc3bcca92e46cb/up-core-api/uprotocol/v1/uattributes.proto#L27-L67):

```
// Metadata describing a particular message's purpose, content and processing requirements.
// Each type of message is described by a set of mandatory and optional attributes.
message UAttributes {
    // A unique message identifier.
    UUID id = 1;


    // This message's type which also indicates its purpose and determines contraints on the other properties.
    UMessageType type = 2;


    // The origin (address) of this message.
    UUri source = 3;


    // The destination (address) of this message.
    UUri sink = 4;

    // ... snip ...
}
```

### Current `UUri`

The 1.6.0-alpha.3 spec version of [`UUri`](https://github.com/eclipse-uprotocol/up-spec/blob/3bbae3786873af72fadf2a6102332f364edc695e/up-core-api/uprotocol/v1/uri.proto#L23-L41) looks like this:

```
// Data model definition for source and destination addressing of messages sent to/from
// devices, services, methods, topics, etc... 
message UUri {
  // Authority Name.
  //
  // Could be the host name, ip address, device & domain names, etc..
  string authority_name = 1;


  // Software Entity (uEntity) Identifiers.
  uint32 ue_id = 2;


  // Software Entity (uEntity) major version number.
  uint32 ue_version_major = 3;


  // uEntity resource id.
  //
  // Identifier used to represent either a method, publish topic, or notification topic.  
  uint32 resource_id = 4;
}
```

### v1.5.7 `UUri`

An earlier version of the uProtocol spec instead allowed greater flexibility in how to represent and support serialization of a `UUri`.

[`UUri`](https://github.com/eclipse-uprotocol/up-core-api/blob/926efe294759b49fbfabbe6713ecab9ef39bbe50/uprotocol/uri.proto#L34-L51)

```
// Data representation of uProtocol <b>URI</b>.
// This class will be used to represent the source and sink (destination) parts 
// of the Packet, for example in a CloudEvent Packet. <br>
// UUri is used as a method to uniquely identify devices, services, and resources
// on the  network.<br>
// Where software is deployed, what the service is called along with a version and
// the resources in the service. Defining a common URI for the system allows 
// applications and/or services to publish and discover each other.<br>
message UUri {
  // Authority of the URI (ex. device name, address, id), if not present, UUri is local
  UAuthority authority = 1;
  
  // Software Entity information (ex. name, version, id)
  UEntity entity = 2;
  
  // Service Resource information (methods, topics, etc..)
  UResource resource = 3;
}
```

The [authority portion](https://github.com/eclipse-uprotocol/up-core-api/blob/926efe294759b49fbfabbe6713ecab9ef39bbe50/uprotocol/uri.proto#L54-L66) was not a string, but instead had an optional string for a name followed by _either_ an IP address or an id:

```
// An Authority represents the deployment location of a specific Software Entity.
// Authority can be represented in either a name (i.e example.com), ip address (205.236.147.1)
// or an ID (i.e. VIN, SHA 128, or any other identifier that is less than 255 bytes.
// *NOTE:* When Authority is empty (neither name, ip, or id are set) the authority
// is local.
message UAuthority {
  optional string name = 1; // Domain & device name as a string
  oneof number {
    bytes ip = 2;       // IPv4 or IPv6 Address in byte format
    bytes id = 3;               // Unique ID for the device, could be a VIN, SHA 128, or any other identifier
                    // *NOTE:* MAX length is 255 bytes
  }
}
```

The [entity portion](https://github.com/eclipse-uprotocol/up-core-api/blob/926efe294759b49fbfabbe6713ecab9ef39bbe50/uprotocol/uri.proto#L69-L82) contained a string for the name of the entity as well as optional fields for `id`, `version_major`, and `version_minor`:

```
// Data representation of an <b> Software Entity - uE</b><br>
// Software entities are distinguished by using a unique name and id 
// along with the specific version of the software.
// An Software Entity is a piece of software deployed somewhere on a uDevice.<br>
// The Software Entity is used in the source and sink parts of communicating software.<br>
// A uE that publishes events is a <b>Service</b> role.<br>
// A uE that consumes events is an <b>Application</b> role.
message UEntity {
  string name = 1;    // Name of the entity
  optional uint32 id = 2;      // The numeric ID for the uEntity assigned from the name/number registry
  optional uint32 version_major = 3; // optional major version of the uEntity
  optional uint32 version_minor = 4; // optional minor version of the uEntity
}
```

Lastly, the [resource portion](https://github.com/eclipse-uprotocol/up-core-api/blob/926efe294759b49fbfabbe6713ecab9ef39bbe50/uprotocol/uri.proto#L84-L98) contained `name`, `instance`, `message`, and `id`:

```
// A service API - defined in the {@link UEntity} - has Resources and Methods. 
// Both of these are represented by the UResource message.
// A uResource represents a resource from a Service such as "door" and an optional
// specific instance such as "front_left". In addition, it can optionally contain
// the name of the resource Message type, such as "Door". The Message type matches
// the protobuf service IDL that defines structured data types.
// An UResource is something that can be manipulated/controlled/exposed by a service.
// Resources are unique when prepended with UAuthority that represents the device and
// UEntity that represents the service. 
message UResource {
  string name = 1;      // Name of the resource
  optional string instance = 2;  // Instance of the resource
  optional string message = 3;   // Message type for the resource
  optional uint32 id = 4;        // Numerical representation of the UResource portion of the UUri
}
```

As you can see, the `UUri` has been vastly simplified from the v1.5.7 version of the spec, but its overall sort of role within uProtocol as serving as a way to specify source and sink has not changed.

We had two concepts at the time: a micro form and a long form. The micro form essentially meant we had to "fill all the numeric fields" to keep the Protobuf on the wire format as compact as possible when communicating with mechatronics devices.

Let's now dig into the issue I saw related to missing validation when serializing the `UUri` into a Protobuf object for the micro form.

## `UUri` serialization validation

As mentioned above, The micro form essentially meant we had to "fill all the numeric fields" such as `UUri::UAuthority::number`, `UUri::UEntity::id`, and `UUri::UResource::id`.

I had just begun work on uProtocol and noticed that there was a bit of a mismatch between how many bits we _expected_ to be in those fields and the _actual_ number of bits in those fields for the micro form. The issue appeared to be due to the smallest representatable unsigned integer in Protobuf's [scalar types](https://protobuf.dev/programming-guides/proto3/#scalar) being a `uint32` and our lack of validation that we were feeding the correct numbers of bits and bytes.

### The state of the validation

The validation at that time was in accordance with the spec, containing functions like [`is_micro_form`](https://github.com/eclipse-uprotocol/up-rust/blob/50b3a7b5d932b351f338518c2e1b7886e9c2da01/src/uri/validator/urivalidator.rs#L195) to check that the fields _existed_ for filling in the micro form.

```rust
    /// Checks if the URI contains numbers so that it can be serialized into micro format.
    ///
    /// # Arguments
    /// * `uri` - The `UUri` to check.
    ///
    /// # Returns
    /// Returns `true` if the URI contains numbers, allowing it to be serialized into micro format.
    #[allow(clippy::missing_panics_doc)]
    pub fn is_micro_form(uri: &UUri) -> bool {
        !Self::is_empty(uri)
            && uri.entity.as_ref().map_or(false, UEntity::has_id)
            && uri.resource.as_ref().map_or(false, UResource::has_id)
            && (uri.authority.is_none()
                || UAuthority::has_ip(uri.authority.as_ref().unwrap())
                || UAuthority::has_id(uri.authority.as_ref().unwrap()))
    }
```

The trouble is that we're merely checking for existence, not ensuring that we have upheld the spec.

I [raised an issue](https://github.com/eclipse-uprotocol/up-rust/issues/11) on `up-rust` to check my understanding. I got confirmation from [Kai Hudalla](https://github.com/sophokles73) that indeed, there was a lack of validation in place to prevent us from doing wacky things like:

* stuffing the `UEntity::id` or `UResource::id` with more bits than allowed
* stufffing the `UEntity::version_major` with more bits than allowed
* stuffing nonsense that is not an IP address into the `UAuthority::number::ip`
* going past the defined limit for length of the `UAuthority::number::id`

So I started to improve our validation by adding guardrails to prevent us from serializing and sending these sorts of malformed `UUri`s.

## The validation improvements

After some great feedback from Kai on how he saw this validation fitting in, I arrived at [this PR](https://github.com/eclipse-uprotocol/up-rust/pull/18) to shore up the validation around the micro form `UUri`.

I'll dive into each improvement made.

### `is_micro_form()`

Here is the updated [`is_micro_form()`](https://github.com/eclipse-uprotocol/up-rust/pull/18/files#diff-50d5281e2de4d6b2de3ffb2294e4a0bb1fe84a474bd1595b080e740cba9e2aacR407) function. We can see that all it does is call the `validate_micro_form()` function and if `Ok(())` is returned then we return true, otherwise return false.

```rust
    pub fn is_micro_form(uri: &UUri) -> bool {
        Self::validate_micro_form(uri).is_ok()
    }
```

### `validate_micro_form()`

Here is [`validate_micro_form()`](https://github.com/eclipse-uprotocol/up-rust/pull/18/files#diff-50d5281e2de4d6b2de3ffb2294e4a0bb1fe84a474bd1595b080e740cba9e2aacR368), where we then call associated functions for each struct that's a member of the `UUri` in order to validate those parts of the `UUri`.

```rust
    pub fn validate_micro_form(uri: &UUri) -> Result<(), ValidationError> {
        if Self::is_empty(uri) {
            Err(ValidationError::new("URI is empty"))?;
        }

        if let Some(entity) = uri.entity.as_ref() {
            if let Err(e) = entity.validate_micro_form() {
                return Err(ValidationError::new(format!("Entity: {}", e)));
            }
        } else {
            return Err(ValidationError::new("Entity: Is missing"));
        }

        if let Some(resource) = uri.resource.as_ref() {
            if let Err(e) = resource.validate_micro_form() {
                return Err(ValidationError::new(format!("Resource: {}", e)));
            }
        } else {
            return Err(ValidationError::new("Resource: Is missing"));
        }

        if let Some(authority) = uri.authority.as_ref() {
            if let Err(e) = authority.validate_micro_form() {
                return Err(ValidationError::new(format!("Authority: {}", e)));
            }
        }

        Ok(())
    }
```

### `UEntity::validate_micro_form()`

[Here](https://github.com/eclipse-uprotocol/up-rust/pull/18/files#diff-11f144d2d2b362e886690dc91f396ffd196c911238f857794ccbc8aa2e04a503R37) we ensure that the `UEntity::id` and `UEntity::version_major` fit within the 16 bits and 8 bits respectively dedicated to them.

Note something powerful we can do here, where we are able to implement a method on `UEntity`, which is the Rust struct created from the .proto object earlier.

Rust lets us use these impl blocks to implement methods (which take `self`) and associated functions (those which don't) in a really seamless way on top of .proto objects.

```rust
const UENTITY_ID_LENGTH: usize = 16;
const UENTITY_ID_VALID_BITMASK: u32 = 0xffff << UENTITY_ID_LENGTH;
const UENTITY_MAJOR_VERSION_LENGTH: usize = 8;
const UENTITY_MAJOR_VERSION_VALID_BITMASK: u32 = 0xffffff << UENTITY_MAJOR_VERSION_LENGTH;

// ... snip ...

impl UEntity {
    // ... snip ...

    /// Returns whether a `UEntity` satisfies the requirements of a micro form URI
    ///
    /// # Returns
    /// Returns a `Result<(), ValidationError>` where the ValidationError will contain the reasons it failed or OK(())
    /// otherwise
    ///
    /// # Errors
    ///
    /// Returns a `ValidationError` in the failure case
    pub fn validate_micro_form(&self) -> Result<(), ValidationError> {
        if let Some(id) = self.id {
            if id & UENTITY_ID_VALID_BITMASK != 0 {
                return Err(ValidationError::new(
                    "ID does not fit within allotted 16 bits in micro form",
                ));
            }
        } else {
            return Err(ValidationError::new("ID must be present"));
        }

        if let Some(major_version) = self.version_major {
            if major_version & UENTITY_MAJOR_VERSION_VALID_BITMASK != 0 {
                return Err(ValidationError::new(
                    "Major version does not fit within 8 allotted bits in micro form",
                ));
            }
        } else {
            return Err(ValidationError::new("Major version must be present"));
        }

        Ok(())
    }
}
```

### `UResource::validate_micro_form()`

The [validation](https://github.com/eclipse-uprotocol/up-rust/pull/18/files#diff-7ddfd85bbae0e72df0c26dfbbd457daed1c52c6dc276284cad38e7faa1bbcb1bR43) for `UResource` is similar, but a little simpler since we need only worry about the `id`.

```rust
const URESOURCE_ID_LENGTH: usize = 16;
const URESOURCE_ID_VALID_BITMASK: u32 = 0xffff << URESOURCE_ID_LENGTH;

impl UResource {

    // ... snip ...
    
    /// Returns whether a `UResource` satisfies the requirements of a micro form URI
    ///
    /// # Returns
    /// Returns a `Result<(), ValidationError>` where the ValidationError will contain the reasons it failed or OK(())
    /// otherwise
    ///
    /// # Errors
    ///
    /// Returns a `ValidationError` in the failure case
    pub fn validate_micro_form(&self) -> Result<(), ValidationError> {
        if let Some(id) = self.id {
            if id & URESOURCE_ID_VALID_BITMASK != 0 {
                return Err(ValidationError::new(
                    "ID does not fit within allotted 16 bits in micro form",
                ));
            }
        } else {
            return Err(ValidationError::new("ID must be present"));
        }

        Ok(())
    } 

}
```

### `UAuthority::validate_micro_form()`

Finally, the [validation](https://github.com/eclipse-uprotocol/up-rust/pull/18/files#diff-f5aec09afbca9d690f4c5a7bd877c4805631d254aa1529f5e06d3a88c123fcd7R39) of the `UAuthority` being in micro form.

There are some neat things going on that I'll highlight below in the code with `PELE`.

```rust
const REMOTE_IPV4_BYTES: usize = 4;
const REMOTE_IPV6_BYTES: usize = 16;
const REMOTE_ID_MINIMUM_BYTES: usize = 1;
const REMOTE_ID_MAXIMUM_BYTES: usize = 255;

impl UAuthority {

    // ... snip ...

    /// Returns whether a `UAuthority` satisfies the requirements of a micro form URI
    ///
    /// # Returns
    /// Returns a `Result<(), ValidationError>` where the ValidationError will contain the reasons it failed or OK(())
    /// otherwise
    ///
    /// # Errors
    ///
    /// Returns a `ValidationError` in the failure case
    pub fn validate_micro_form(&self) -> Result<(), ValidationError> {
        let Some(number) = &self.number else {
            return Err(ValidationError::new(
                "Must have IP address or ID set as UAuthority for micro form. Neither are set.",
            ));
        };

        // PELE: Here we can make use of Rust's match expressions + early return
        //   to check for erroneous state
        match number {
            Number::Ip(ip) => {
                // PELE: We confirm here that _at a minimum_ the number of bytes matches
                //   either the IPv4 or IPv6 spec
                if !(ip.len() == REMOTE_IPV4_BYTES || ip.len() == REMOTE_IPV6_BYTES) {
                    return Err(ValidationError::new(
                        "IP address is not IPv4 (4 bytes) or IPv6 (16 bytes)",
                    ));
                }
            }
            Number::Id(id) => {
                // PELE: Here we ensure that the length of the `id` is between
                //   REMOTE_ID_MINIMUM_BYTES (1) and REMOTE_ID_MAXIMUM_BYTES (255)
                //   inclusive by way of the foo..=bar construct
                if !matches!(id.len(), REMOTE_ID_MINIMUM_BYTES..=REMOTE_ID_MAXIMUM_BYTES) {
                    return Err(ValidationError::new("ID doesn't fit in bytes allocated"));
                }
            }
        }
        Ok(())
    }

}
```

### Adding unit tests

After adding these additional validation checks, some unit tests failed, and I found that some unit test gaps were more obvious.

I then went through and added doc tests to ensure that I had captured the logic correctly, such as [this one](https://github.com/eclipse-uprotocol/up-rust/pull/18/files#diff-50d5281e2de4d6b2de3ffb2294e4a0bb1fe84a474bd1595b080e740cba9e2aacR268-R296):

```rust
    /// ## `UAuthority` ID is longer than maximum allowed (255 bytes)
    /// use up_rust::uprotocol::{UAuthority, UUri, UEntity, UResource, uri::uauthority::Number};
    /// use up_rust::uri::validator::{UriValidator, ValidationError};
    ///
    /// let uri = UUri {
    ///     authority: Some(UAuthority {
    ///         number: Some(Number::Ip(
    ///                 (0..=256) // <- note that ID will exceed 255 byte limit
    ///                 .map(|i| (i % 256) as u8)
    ///                 .collect::<Vec<u8>>())),
    ///         ..Default::default()
    ///     })
    ///     .into(),
    ///     entity: Some(UEntity {
    ///         id: Some(29999),
    ///         version_major: Some(254),
    ///         ..Default::default()
    ///     })
    ///     .into(),
    ///     resource: Some(UResource {
    ///         id: Some(29999),
    ///         ..Default::default()
    ///     })
    ///     .into(),
    ///     ..Default::default()
    /// };
    /// let val_micro_form = UriValidator::validate_micro_form(&uri);
    /// assert!(val_micro_form.is_err());
```

I love Rust's documentation system. By putting the above in `///` above the function then `cargo doc` will generate HTML documentation which shows off these examples.

_In addition_, because these are boxed in a ticked block "```", these examples cannot go stale. They are checked and run by the compiler and should one of these sections fail to compile or fail an invariant in the test, then we'd immediately be alerted to a breaking change we had made.

## And that's a wrap

The more rigorous validation I put in place ended up being a success, so I also [opened](https://github.com/eclipse-uprotocol/up-spec/pull/78) up [several](https://github.com/eclipse-uprotocol/up-spec/pull/94) [PR](https://github.com/eclipse-uprotocol/up-spec/pull/101)s to then enforce that the other language libraries such as Java and C++ would also follow suit.

We eventually migrated to the current version of the `UUri`, but I'll save those details for another day.

Thanks and till next time! 👋

