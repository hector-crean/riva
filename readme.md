
For testing, run:
`surreal start --user root --pass root memory`




Liveblocks and Livekit provide the infrasturcutre for realtime colaborative applications. 




Zed is a realtime collaborative editor 

```

```




Yes, exactly. When a client receives the StorageUpdated event over the WebSocket connection, it would typically:
Recognize that the shared document state has changed
Make an HTTP request (usually a GET request) to a dedicated API endpoint to fetch the latest storage state for that specific room
Update its local state with the retrieved data
This pattern separates concerns effectively:
WebSockets handle real-time notifications and lightweight events
HTTP requests handle the transfer of potentially larger state data
This approach has several advantages:
Reduces WebSocket message size and frequency
Allows clients to decide when to fetch updates (they might batch or debounce requests if many updates happen quickly)
Provides a clear separation between notification of changes and the actual data transfer
Enables more efficient caching strategies on the HTTP layer



1. Active Room State
In-memory state: Active rooms are kept in memory for fast access and real-time operations
Connection information: Which users are connected to which rooms
Presence data: Real-time user presence information (cursor positions, selection states, etc.)

2. Persistent Storage
Document data: The actual collaborative content (stored in their database)
Room metadata: Information about rooms, permissions, etc.
Historical changes: For conflict resolution and history tracking



