using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Numerics;

namespace riva_ws_server {

    public sealed class RoomLeftPayload: IEquatable<RoomLeftPayload>, ICloneable {
        public string user_id;

        public RoomLeftPayload(string _user_id) {
            if (_user_id == null) throw new ArgumentNullException(nameof(_user_id));
            user_id = _user_id;
        }

        public void Serialize(Serde.ISerializer serializer) {
            serializer.increase_container_depth();
            serializer.serialize_str(user_id);
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

        public static RoomLeftPayload Deserialize(Serde.IDeserializer deserializer) {
            deserializer.increase_container_depth();
            RoomLeftPayload obj = new RoomLeftPayload(
            	deserializer.deserialize_str());
            deserializer.decrease_container_depth();
            return obj;
        }

        public static RoomLeftPayload BincodeDeserialize(byte[] input) => BincodeDeserialize(new ArraySegment<byte>(input));

        public static RoomLeftPayload BincodeDeserialize(ArraySegment<byte> input) {
            if (input == null) {
                 throw new Serde.DeserializationException("Cannot deserialize null array");
            }
            Serde.IDeserializer deserializer = new Bincode.BincodeDeserializer(input);
            RoomLeftPayload value = Deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.Count) {
                 throw new Serde.DeserializationException("Some input bytes were not read");
            }
            return value;
        }
        public override bool Equals(object obj) => obj is RoomLeftPayload other && Equals(other);

        public static bool operator ==(RoomLeftPayload left, RoomLeftPayload right) => Equals(left, right);

        public static bool operator !=(RoomLeftPayload left, RoomLeftPayload right) => !Equals(left, right);

        public bool Equals(RoomLeftPayload other) {
            if (other == null) return false;
            if (ReferenceEquals(this, other)) return true;
            if (!user_id.Equals(other.user_id)) return false;
            return true;
        }

        public override int GetHashCode() {
            unchecked {
                int value = 7;
                value = 31 * value + user_id.GetHashCode();
                return value;
            }
        }

        /// <summary>Creates a shallow clone of the object.</summary>
        public RoomLeftPayload Clone() => (RoomLeftPayload)MemberwiseClone();

        object ICloneable.Clone() => Clone();

    }

} // end of namespace riva_ws_server
