1. As a **user**, I want to search for cards from all cards produced.
  - So that I can find details about cards
  - [AC]: Cards can be searched by their name
  
2. As a **user**, I want to select which cards are in my collection.
  - So that I can keep track and manage of my collection
  - [AC]: Cards can be selected from the pool of all cards and added to collection
  
3. As a **user**, I want to make decks
  - So that I can make cohesive subgroups from a large collection
  - [AC]: Simple CRUD operations on decks
  
4. As a **collector of rarer cards**, I want to select specific printings 
  - So that I can make my virtual collection's visual appearance match the physical
  - [AC]: Printings can be selected; visual appearance updates based on selection
  
5. As a **competitive player**, I want to make certain decks "exclusive"
  - So that I can easily see which cards are available for future decks.
  - [AC]: A deck can be toggled to exclusive. Afterwards, its cards aren't available for other decks. The operation fails if any cards are shared in other decks.
  
6. As a **noncompetitive player**, I want to make certain decks "non-exclusive"
  - So that I can easily share generic cards across decks.
  - [AC]: A deck can be toggled to nonexclusive. Afterwards, its cards _are_ available for other _nonexclusive_ decks
  
7. As a **long-time player & collector**, I want to manage a large collection of cards
  - So that I can grow my collection to any size without worrying about lag 
  - [AC]: [NFR]
  
8. As a **long-time player & collector**, I want to import files from existing deck-management services
  - So that I don't _have_ to manually select my large collection
  - [AC]: .dek files can be imported
  
9. As a **newer player**, I want to search and filter based on card criteria
  - So that I can find a card that matches what I want
  - [AC]: All groups (deck, collection, pool of all cards) can be searched based on all fields of card
  
10. As a **nerdy Magic player** I want my collection to be stored on my own computer
  - So that I have data ownership
  - [AC]: Collection is stored in files on user's computer
  
11. As a **non-nerdy Magic player**, I want my collection to be able to sync across devices
  - So that I can access my collection on-the-go
  - [AC]: [STRETCH] Collection can be synced with online service 
  
12. As a **user**, I want to know that the software will handle all cards I have
  - So that I don't fear importing most of my cards and then running up against impossibilities in the software
  - [AC]: [NFR]
  
