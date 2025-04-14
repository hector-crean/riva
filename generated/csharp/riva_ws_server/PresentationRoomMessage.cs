using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Numerics;

namespace riva_ws_server {

    public abstract class PresentationRoomMessage: IEquatable<PresentationRoomMessage>, ICloneable {

        public abstract void Serialize(Serde.ISerializer serializer);

        public static PresentationRoomMessage Deserialize(Serde.IDeserializer deserializer) {
            int index = deserializer.deserialize_variant_index();
            switch (index) {
                case 0: return JoinRoom.Load(deserializer);
                case 1: return LeaveRoom.Load(deserializer);
                case 2: return RequestSlideChange.Load(deserializer);
                case 3: return RoomJoined.Load(deserializer);
                case 4: return RoomLeft.Load(deserializer);
                case 5: return SlideChanged.Load(deserializer);
                default: throw new Serde.DeserializationException("Unknown variant index for PresentationRoomMessage: " + index);
            }
        }

        public int BincodeSerialize(byte[] outputBuffer) => BincodeSerialize(new ArraySegment<byte>(outputBuffer));

        public int BincodeSerialize(ArraySegment<byte> outputBuffer) {
            Serde.ISerializer serializer = new Bincode.BincodeSerializer(outputBuffer);
            Serialize(serializer);
            return serializer.get_buffer_offset();
        }

        public byte[] BincodeSerialize()  {
            Serde.ISerializer serializer = new Bincode.BincodeSerializer();
            Serialize(serializer);
            return serializer.get_bytes();
        }

        public static PresentationRoomMessage BincodeDeserialize(byte[] input) => BincodeDeserialize(new ArraySegment<byte>(input));

        public static PresentationRoomMessage BincodeDeserialize(ArraySegment<byte> input) {
            if (input == null) {
                 throw new Serde.DeserializationException("Cannot deserialize null array");
            }
            Serde.IDeserializer deserializer = new Bincode.BincodeDeserializer(input);
            PresentationRoomMessage value = Deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.Count) {
                 throw new Serde.DeserializationException("Some input bytes were not read");
            }
            return value;
        }
        public override int GetHashCode() {
            switch (this) {
            case JoinRoom x: return x.GetHashCode();
            case LeaveRoom x: return x.GetHashCode();
            case RequestSlideChange x: return x.GetHashCode();
            case RoomJoined x: return x.GetHashCode();
            case RoomLeft x: return x.GetHashCode();
            case SlideChanged x: return x.GetHashCode();
            default: throw new InvalidOperationException("Unknown variant type");
            }
        }
        public override bool Equals(object obj) => obj is PresentationRoomMessage other && Equals(other);

        public bool Equals(PresentationRoomMessage other) {
            if (other == null) return false;
            if (ReferenceEquals(this, other)) return true;
            if (GetType() != other.GetType()) return false;
            switch (this) {
            case JoinRoom x: return x.Equals((JoinRoom)other);
            case LeaveRoom x: return x.Equals((LeaveRoom)other);
            case RequestSlideChange x: return x.Equals((RequestSlideChange)other);
            case RoomJoined x: return x.Equals((RoomJoined)other);
            case RoomLeft x: return x.Equals((RoomLeft)other);
            case SlideChanged x: return x.Equals((SlideChanged)other);
            default: throw new InvalidOperationException("Unknown variant type");
            }
        }

        /// <summary>Creates a shallow clone of the object.</summary>
        public PresentationRoomMessage Clone() => (PresentationRoomMessage)MemberwiseClone();

        object ICloneable.Clone() => Clone();


        public sealed class JoinRoom: PresentationRoomMessage, IEquatable<JoinRoom>, ICloneable {
            public JoinRoomPayload value;

            public JoinRoom(JoinRoomPayload _value) {
                if (_value == null) throw new ArgumentNullException(nameof(_value));
                value = _value;
            }

            public override void Serialize(Serde.ISerializer serializer) {
                serializer.increase_container_depth();
                serializer.serialize_variant_index(0);
                value.Serialize(serializer);
                serializer.decrease_container_depth();
            }

            internal static JoinRoom Load(Serde.IDeserializer deserializer) {
                deserializer.increase_container_depth();
                JoinRoom obj = new JoinRoom(
                	JoinRoomPayload.Deserialize(deserializer));
                deserializer.decrease_container_depth();
                return obj;
            }
            public override bool Equals(object obj) => obj is JoinRoom other && Equals(other);

            public static bool operator ==(JoinRoom left, JoinRoom right) => Equals(left, right);

            public static bool operator !=(JoinRoom left, JoinRoom right) => !Equals(left, right);

            public bool Equals(JoinRoom other) {
                if (other == null) return false;
                if (ReferenceEquals(this, other)) return true;
                if (!value.Equals(other.value)) return false;
                return true;
            }

            public override int GetHashCode() {
                unchecked {
                    int value = 7;
                    value = 31 * value + value.GetHashCode();
                    return value;
                }
            }

        }

        public sealed class LeaveRoom: PresentationRoomMessage, IEquatable<LeaveRoom>, ICloneable {
            public LeaveRoomPayload value;

            public LeaveRoom(LeaveRoomPayload _value) {
                if (_value == null) throw new ArgumentNullException(nameof(_value));
                value = _value;
            }

            public override void Serialize(Serde.ISerializer serializer) {
                serializer.increase_container_depth();
                serializer.serialize_variant_index(1);
                value.Serialize(serializer);
                serializer.decrease_container_depth();
            }

            internal static LeaveRoom Load(Serde.IDeserializer deserializer) {
                deserializer.increase_container_depth();
                LeaveRoom obj = new LeaveRoom(
                	LeaveRoomPayload.Deserialize(deserializer));
                deserializer.decrease_container_depth();
                return obj;
            }
            public override bool Equals(object obj) => obj is LeaveRoom other && Equals(other);

            public static bool operator ==(LeaveRoom left, LeaveRoom right) => Equals(left, right);

            public static bool operator !=(LeaveRoom left, LeaveRoom right) => !Equals(left, right);

            public bool Equals(LeaveRoom other) {
                if (other == null) return false;
                if (ReferenceEquals(this, other)) return true;
                if (!value.Equals(other.value)) return false;
                return true;
            }

            public override int GetHashCode() {
                unchecked {
                    int value = 7;
                    value = 31 * value + value.GetHashCode();
                    return value;
                }
            }

        }

        public sealed class RequestSlideChange: PresentationRoomMessage, IEquatable<RequestSlideChange>, ICloneable {
            public RequestSlideChangePayload value;

            public RequestSlideChange(RequestSlideChangePayload _value) {
                if (_value == null) throw new ArgumentNullException(nameof(_value));
                value = _value;
            }

            public override void Serialize(Serde.ISerializer serializer) {
                serializer.increase_container_depth();
                serializer.serialize_variant_index(2);
                value.Serialize(serializer);
                serializer.decrease_container_depth();
            }

            internal static RequestSlideChange Load(Serde.IDeserializer deserializer) {
                deserializer.increase_container_depth();
                RequestSlideChange obj = new RequestSlideChange(
                	RequestSlideChangePayload.Deserialize(deserializer));
                deserializer.decrease_container_depth();
                return obj;
            }
            public override bool Equals(object obj) => obj is RequestSlideChange other && Equals(other);

            public static bool operator ==(RequestSlideChange left, RequestSlideChange right) => Equals(left, right);

            public static bool operator !=(RequestSlideChange left, RequestSlideChange right) => !Equals(left, right);

            public bool Equals(RequestSlideChange other) {
                if (other == null) return false;
                if (ReferenceEquals(this, other)) return true;
                if (!value.Equals(other.value)) return false;
                return true;
            }

            public override int GetHashCode() {
                unchecked {
                    int value = 7;
                    value = 31 * value + value.GetHashCode();
                    return value;
                }
            }

        }

        public sealed class RoomJoined: PresentationRoomMessage, IEquatable<RoomJoined>, ICloneable {
            public RoomJoinedPayload value;

            public RoomJoined(RoomJoinedPayload _value) {
                if (_value == null) throw new ArgumentNullException(nameof(_value));
                value = _value;
            }

            public override void Serialize(Serde.ISerializer serializer) {
                serializer.increase_container_depth();
                serializer.serialize_variant_index(3);
                value.Serialize(serializer);
                serializer.decrease_container_depth();
            }

            internal static RoomJoined Load(Serde.IDeserializer deserializer) {
                deserializer.increase_container_depth();
                RoomJoined obj = new RoomJoined(
                	RoomJoinedPayload.Deserialize(deserializer));
                deserializer.decrease_container_depth();
                return obj;
            }
            public override bool Equals(object obj) => obj is RoomJoined other && Equals(other);

            public static bool operator ==(RoomJoined left, RoomJoined right) => Equals(left, right);

            public static bool operator !=(RoomJoined left, RoomJoined right) => !Equals(left, right);

            public bool Equals(RoomJoined other) {
                if (other == null) return false;
                if (ReferenceEquals(this, other)) return true;
                if (!value.Equals(other.value)) return false;
                return true;
            }

            public override int GetHashCode() {
                unchecked {
                    int value = 7;
                    value = 31 * value + value.GetHashCode();
                    return value;
                }
            }

        }

        public sealed class RoomLeft: PresentationRoomMessage, IEquatable<RoomLeft>, ICloneable {
            public RoomLeftPayload value;

            public RoomLeft(RoomLeftPayload _value) {
                if (_value == null) throw new ArgumentNullException(nameof(_value));
                value = _value;
            }

            public override void Serialize(Serde.ISerializer serializer) {
                serializer.increase_container_depth();
                serializer.serialize_variant_index(4);
                value.Serialize(serializer);
                serializer.decrease_container_depth();
            }

            internal static RoomLeft Load(Serde.IDeserializer deserializer) {
                deserializer.increase_container_depth();
                RoomLeft obj = new RoomLeft(
                	RoomLeftPayload.Deserialize(deserializer));
                deserializer.decrease_container_depth();
                return obj;
            }
            public override bool Equals(object obj) => obj is RoomLeft other && Equals(other);

            public static bool operator ==(RoomLeft left, RoomLeft right) => Equals(left, right);

            public static bool operator !=(RoomLeft left, RoomLeft right) => !Equals(left, right);

            public bool Equals(RoomLeft other) {
                if (other == null) return false;
                if (ReferenceEquals(this, other)) return true;
                if (!value.Equals(other.value)) return false;
                return true;
            }

            public override int GetHashCode() {
                unchecked {
                    int value = 7;
                    value = 31 * value + value.GetHashCode();
                    return value;
                }
            }

        }

        public sealed class SlideChanged: PresentationRoomMessage, IEquatable<SlideChanged>, ICloneable {
            public SlideChangedPayload value;

            public SlideChanged(SlideChangedPayload _value) {
                if (_value == null) throw new ArgumentNullException(nameof(_value));
                value = _value;
            }

            public override void Serialize(Serde.ISerializer serializer) {
                serializer.increase_container_depth();
                serializer.serialize_variant_index(5);
                value.Serialize(serializer);
                serializer.decrease_container_depth();
            }

            internal static SlideChanged Load(Serde.IDeserializer deserializer) {
                deserializer.increase_container_depth();
                SlideChanged obj = new SlideChanged(
                	SlideChangedPayload.Deserialize(deserializer));
                deserializer.decrease_container_depth();
                return obj;
            }
            public override bool Equals(object obj) => obj is SlideChanged other && Equals(other);

            public static bool operator ==(SlideChanged left, SlideChanged right) => Equals(left, right);

            public static bool operator !=(SlideChanged left, SlideChanged right) => !Equals(left, right);

            public bool Equals(SlideChanged other) {
                if (other == null) return false;
                if (ReferenceEquals(this, other)) return true;
                if (!value.Equals(other.value)) return false;
                return true;
            }

            public override int GetHashCode() {
                unchecked {
                    int value = 7;
                    value = 31 * value + value.GetHashCode();
                    return value;
                }
            }

        }
    }


} // end of namespace riva_ws_server
