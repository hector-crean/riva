using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Numerics;

namespace riva_ws_server {

    public sealed class JoinRoomPayload: IEquatable<JoinRoomPayload>, ICloneable {
        public RoomId room_id;

        public JoinRoomPayload(RoomId _room_id) {
            if (_room_id == null) throw new ArgumentNullException(nameof(_room_id));
            room_id = _room_id;
        }

        public void Serialize(Serde.ISerializer serializer) {
            serializer.increase_container_depth();
            room_id.Serialize(serializer);
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

        public static JoinRoomPayload Deserialize(Serde.IDeserializer deserializer) {
            deserializer.increase_container_depth();
            JoinRoomPayload obj = new JoinRoomPayload(
            	RoomId.Deserialize(deserializer));
            deserializer.decrease_container_depth();
            return obj;
        }

        public static JoinRoomPayload BincodeDeserialize(byte[] input) => BincodeDeserialize(new ArraySegment<byte>(input));

        public static JoinRoomPayload BincodeDeserialize(ArraySegment<byte> input) {
            if (input == null) {
                 throw new Serde.DeserializationException("Cannot deserialize null array");
            }
            Serde.IDeserializer deserializer = new Bincode.BincodeDeserializer(input);
            JoinRoomPayload value = Deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.Count) {
                 throw new Serde.DeserializationException("Some input bytes were not read");
            }
            return value;
        }
        public override bool Equals(object obj) => obj is JoinRoomPayload other && Equals(other);

        public static bool operator ==(JoinRoomPayload left, JoinRoomPayload right) => Equals(left, right);

        public static bool operator !=(JoinRoomPayload left, JoinRoomPayload right) => !Equals(left, right);

        public bool Equals(JoinRoomPayload other) {
            if (other == null) return false;
            if (ReferenceEquals(this, other)) return true;
            if (!room_id.Equals(other.room_id)) return false;
            return true;
        }

        public override int GetHashCode() {
            unchecked {
                int value = 7;
                value = 31 * value + room_id.GetHashCode();
                return value;
            }
        }

        /// <summary>Creates a shallow clone of the object.</summary>
        public JoinRoomPayload Clone() => (JoinRoomPayload)MemberwiseClone();

        object ICloneable.Clone() => Clone();

    }

} // end of namespace riva_ws_server
