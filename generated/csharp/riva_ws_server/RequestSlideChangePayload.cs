using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Numerics;

namespace riva_ws_server {

    public sealed class RequestSlideChangePayload: IEquatable<RequestSlideChangePayload>, ICloneable {
        public ulong slide_index;

        public RequestSlideChangePayload(ulong _slide_index) {
            slide_index = _slide_index;
        }

        public void Serialize(Serde.ISerializer serializer) {
            serializer.increase_container_depth();
            serializer.serialize_u64(slide_index);
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

        public static RequestSlideChangePayload Deserialize(Serde.IDeserializer deserializer) {
            deserializer.increase_container_depth();
            RequestSlideChangePayload obj = new RequestSlideChangePayload(
            	deserializer.deserialize_u64());
            deserializer.decrease_container_depth();
            return obj;
        }

        public static RequestSlideChangePayload BincodeDeserialize(byte[] input) => BincodeDeserialize(new ArraySegment<byte>(input));

        public static RequestSlideChangePayload BincodeDeserialize(ArraySegment<byte> input) {
            if (input == null) {
                 throw new Serde.DeserializationException("Cannot deserialize null array");
            }
            Serde.IDeserializer deserializer = new Bincode.BincodeDeserializer(input);
            RequestSlideChangePayload value = Deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.Count) {
                 throw new Serde.DeserializationException("Some input bytes were not read");
            }
            return value;
        }
        public override bool Equals(object obj) => obj is RequestSlideChangePayload other && Equals(other);

        public static bool operator ==(RequestSlideChangePayload left, RequestSlideChangePayload right) => Equals(left, right);

        public static bool operator !=(RequestSlideChangePayload left, RequestSlideChangePayload right) => !Equals(left, right);

        public bool Equals(RequestSlideChangePayload other) {
            if (other == null) return false;
            if (ReferenceEquals(this, other)) return true;
            if (!slide_index.Equals(other.slide_index)) return false;
            return true;
        }

        public override int GetHashCode() {
            unchecked {
                int value = 7;
                value = 31 * value + slide_index.GetHashCode();
                return value;
            }
        }

        /// <summary>Creates a shallow clone of the object.</summary>
        public RequestSlideChangePayload Clone() => (RequestSlideChangePayload)MemberwiseClone();

        object ICloneable.Clone() => Clone();

    }

} // end of namespace riva_ws_server
