using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Numerics;

namespace riva_ws_server {

    public sealed class LeaveRoomPayload: IEquatable<LeaveRoomPayload>, ICloneable {
        public string room;

        public LeaveRoomPayload(string _room) {
            if (_room == null) throw new ArgumentNullException(nameof(_room));
            room = _room;
        }

        public void Serialize(Serde.ISerializer serializer) {
            serializer.increase_container_depth();
            serializer.serialize_str(room);
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

        public static LeaveRoomPayload Deserialize(Serde.IDeserializer deserializer) {
            deserializer.increase_container_depth();
            LeaveRoomPayload obj = new LeaveRoomPayload(
            	deserializer.deserialize_str());
            deserializer.decrease_container_depth();
            return obj;
        }

        public static LeaveRoomPayload BincodeDeserialize(byte[] input) => BincodeDeserialize(new ArraySegment<byte>(input));

        public static LeaveRoomPayload BincodeDeserialize(ArraySegment<byte> input) {
            if (input == null) {
                 throw new Serde.DeserializationException("Cannot deserialize null array");
            }
            Serde.IDeserializer deserializer = new Bincode.BincodeDeserializer(input);
            LeaveRoomPayload value = Deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.Count) {
                 throw new Serde.DeserializationException("Some input bytes were not read");
            }
            return value;
        }
        public override bool Equals(object obj) => obj is LeaveRoomPayload other && Equals(other);

        public static bool operator ==(LeaveRoomPayload left, LeaveRoomPayload right) => Equals(left, right);

        public static bool operator !=(LeaveRoomPayload left, LeaveRoomPayload right) => !Equals(left, right);

        public bool Equals(LeaveRoomPayload other) {
            if (other == null) return false;
            if (ReferenceEquals(this, other)) return true;
            if (!room.Equals(other.room)) return false;
            return true;
        }

        public override int GetHashCode() {
            unchecked {
                int value = 7;
                value = 31 * value + room.GetHashCode();
                return value;
            }
        }

        /// <summary>Creates a shallow clone of the object.</summary>
        public LeaveRoomPayload Clone() => (LeaveRoomPayload)MemberwiseClone();

        object ICloneable.Clone() => Clone();

    }

} // end of namespace riva_ws_server
