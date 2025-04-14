using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Numerics;

namespace riva_ws_server {

    public sealed class RoomId: IEquatable<RoomId>, ICloneable {
        public string room_name;
        public string organisation_id;

        public RoomId(string _room_name, string _organisation_id) {
            if (_room_name == null) throw new ArgumentNullException(nameof(_room_name));
            room_name = _room_name;
            if (_organisation_id == null) throw new ArgumentNullException(nameof(_organisation_id));
            organisation_id = _organisation_id;
        }

        public void Serialize(Serde.ISerializer serializer) {
            serializer.increase_container_depth();
            serializer.serialize_str(room_name);
            serializer.serialize_str(organisation_id);
            serializer.decrease_container_depth();
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

        public static RoomId Deserialize(Serde.IDeserializer deserializer) {
            deserializer.increase_container_depth();
            RoomId obj = new RoomId(
            	deserializer.deserialize_str(),
            	deserializer.deserialize_str());
            deserializer.decrease_container_depth();
            return obj;
        }

        public static RoomId BincodeDeserialize(byte[] input) => BincodeDeserialize(new ArraySegment<byte>(input));

        public static RoomId BincodeDeserialize(ArraySegment<byte> input) {
            if (input == null) {
                 throw new Serde.DeserializationException("Cannot deserialize null array");
            }
            Serde.IDeserializer deserializer = new Bincode.BincodeDeserializer(input);
            RoomId value = Deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.Count) {
                 throw new Serde.DeserializationException("Some input bytes were not read");
            }
            return value;
        }
        public override bool Equals(object obj) => obj is RoomId other && Equals(other);

        public static bool operator ==(RoomId left, RoomId right) => Equals(left, right);

        public static bool operator !=(RoomId left, RoomId right) => !Equals(left, right);

        public bool Equals(RoomId other) {
            if (other == null) return false;
            if (ReferenceEquals(this, other)) return true;
            if (!room_name.Equals(other.room_name)) return false;
            if (!organisation_id.Equals(other.organisation_id)) return false;
            return true;
        }

        public override int GetHashCode() {
            unchecked {
                int value = 7;
                value = 31 * value + room_name.GetHashCode();
                value = 31 * value + organisation_id.GetHashCode();
                return value;
            }
        }

        /// <summary>Creates a shallow clone of the object.</summary>
        public RoomId Clone() => (RoomId)MemberwiseClone();

        object ICloneable.Clone() => Clone();

    }

} // end of namespace riva_ws_server
