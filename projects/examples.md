https://samphina.com.ng/network-programming-secured-client-secured-chat-application/
Computer Science Project Material with Source Code Included

Abstract
Several network systems are built to communicate with one another as well as made available through service-oriented architectures. In this project, the client server architecture is used to develop a chat application. Firstly a chat application is created for both Client and Server which is based on Transmission Control Protocol (TCP) where TCP is connection oriented protocol and is a reliable connection protocol. As security is the key factor while communicating over a network, so in this project, MySQL SSL protocol and hash function was used for the Database based on a numbers of benefits. The hash values of the real password and the random generated number (salt) is stored in the database. The original password is not stored on the system, making cracking of password much harder.

Table Of Content
Title Page
Certification
Approval Page
Dedication
Acknowledgement
Abstract
Table of Contents
List of Tables
List of Figures
Chapter One:
Introduction
1.1 Introduction
1.2 Background of the study
1.3 Statement of the problem
1.4 Objectives of the study
1.5 Significance of the study
1.6 Scope of the study
1.7 Limitations
1.8 Organization of the work
1.9 Definition of terms
Chapter Two:
2.0 Literature Review
2.1 Client-Server and other models
2.2 Client-Server communication
2.3 Host identification and service port
2.4 Sockets and socket based communication
2.5 TCP/IP Socket programming
2.6 Socket programming in Java
2.7 Secure internet programming
2.8 Overview of secure socket layer (SSL)
2.9 Security
2.10 Hash functions
Chapter Three:
3.0 System Analysis And Design
3.1 Methodology
3.2 Primary Data collection
3.2.1 Secondary Data collection
3.3 Analysis of the existing system
3.4 Limitations of the existing system
3.5 System Design
3.6 Database Design
3.7 System Flowchart
3.8 Top Down Diagram
3.9 Justification of the new system
Chapter Four:
4.0 Implementation Testing And Integration
4.1 Choice of development tools
4.2 System Requirements
4.2.1 Software Requirements
4.2.2 Hardware Requirements
4.3 Implementation
4.4 Testing
4.4.1 Unit Test
4.4.2 System Test
4.5 Integration
Chapter Five:
5.0 Summary, Recommendations And Conclusion
5.1 Summary
5.2 Limitations
5.3 Recommendations
5.4 Bill Of Engineering Measurement And Evaluation
5.3 Conclusion
Bibliography
Appendix A: Program Codes
BAChatClient.java
BAChatServer.java
DatabaseManager.java
Encryptor.java
Appendix B: Sample Output
Appendix C: User Guid
Chapter One
Introduction
1.1 Introduction
Several network systems are built to communicate with one another and are made available through service-oriented architectures. In this project, we use the client server architecture to develop a secured Client-Server chat application. A chat application is created based on Transmission Control Protocol (TCP) where TCP is connection oriented protocol and in the end, multithreading is used to develop the application.

A client-server chat application consists of a Chat Client and a Chat Server and there exists a two way communication between them. Here, Message Processor is used to interpret message from the user, Message Interpreter is used to extract and pass the received message. Message Maker is used to construct back the message and Client Manager is used to maintain the clients list which the sender and receiver at both sides use to interact with each other.

In general, the server process will start on some computer system; in fact, the server should be executed before the client. Server usually initializes itself, and then goes to wait state or sleep state where it will wait for a client request. After that, a client process can start on either the same machine or on some other machine. Whenever the client wants some service from the server, it will send a request to the server and the server will accept the request and process it. After the server has finished providing its service to the client, the server will again go back to sleep, that is, waiting for the next client request to arrive. This process is repeated as long as the server processes is running. Whenever such request comes, the server can immediately serve the client and again go back to the waiting state for the next request to arrive.

1.2 Background of the Study
Client server model is the standard model which has been accepted by many for developing network applications. In this model, there is a notion of client and notion of server. As the name implies, a server is a process (or a computer in which the process is running) that is offering some services to other entities which are called clients. A client on the other hand is process (which is running) on the same computer or other computer that is requesting the services provided by the server.

A chat application is basically a combination of two applications:

Server application
Client application
Server application runs on the server computer and client application runs on the client computer (or the machine with server). In this chat application, a client can send data to anyone who is connected to the server.

Java application programming interface (API) provides the classes for creating sockets to facilitate program communications over the network. Sockets are the endpoints of logical connections between two hosts and can be used to send and receive data. Java treats socket communications much as it treat input and output operations; thus programs can read from or write to sockets as easily as they can read from or write to files.

To establish a server connection, a server socket needs to be created and attached to a port, which is where the server listens for connections. The port recognizes the Transmission Control Protocol service on the socket. For instance, the email server runs on port 25, and the web server usually runs on port 80.

Server Execution: At server the side, a thread is created which receives numerous clients’ requests. It also contains a list in which Client’s name and IP addresses are stored. After that, it broadcast the list to all the users who are currently in chat room and when a client logs out then server deletes that particular client from the list, update the list and then broadcast the list to all available clients.

Client Execution: A client firstly must have to register itself by sending username to the server and should have to start the thread so that system can get the list of all available clients. Then any of two registered clients can communicate with each other.

1.3 Statement of the Problem
The client-server communication model is used in a wide variety of software applications. Where normally the server side is sufficiently protected and sealed from public access, but client applications running on devices like notebooks and desktops are considered insecure and exposed to security threats.

The main weakness of client-server chat application is that there is no security provided to data which is transferred between clients. Any unauthorized client can hack the client account and can change the data. This is the main objective of this project (To develop a secured Client-Server Chat Application).

1.4 Objectives of the Study
The aim of this project is to develop a reliable and secure network programming (Client-Server chat model) which can perform a multithreaded server client chat application based on Java socket programming using Transport Control Protocol (TCP). As security is the key factor while communicating over a network, hash function with salt is used for the Database based on a number of benefits. MySQL became the choice for the implementation of this application based on its scalability and flexibility, high performance, high availability, strong data protection, web and data warehouse strengths, management ease, lowest total cost of ownership and open source freedom.

1.5 Significance of the Study
Apart from just performing the regular client server chat, this client-server chat is robust and significant in the following ways: This project use MySQL for its database to make information in the database secure. The personal details and messages including the private messages in the Database are encrypted using encryptor (one of the security facilities available in the MySQL).

This project implements hash function with the password before the encryption and then stored in the Database. It also uses random generated numbers (salt) that is calculated together with the passworded hash values and stored in the Database. As a result, even if the database is compromised, the salt added to hash values makes it harder to compute the original password.

This random salt is used with the hash function to significantly increase the strength of encrypting passwords and thus makes cracking greatly impossible. This makes the chat application server reliable and more secured. Another significance of this application is private chatting. This is where two users can chat in private. The messages between the users are not displayed / seen in the general chat display text field. The messages are displayed only within the private message display text field.

1.6 Scope of the Study
The project shall consider among other things the following issues:

To provide a better understanding of how network programming in java works.
Develop a reliable network communication for a Client-Server chat application.
Analyses of network programming in java (Multithreaded Client-Server Chat applications) for better understanding of the solutions.
Conduct an experimental result in order to establish the parameter of the problem. In conclusion, suggest ways the problems can be eliminated and recommends how the problems can be prevented.
1.7 Limitations
The previous Client-Server Chat system implements only hash function with the password before the encryption which is then stored in the Database. Thus, the database can be compromised easily to compute the original password.

Some drawbacks of the Client-Server Chat are as follows:

As the server receives as many requests from clients so there is a chance that server can become congested and overloaded.
In case of server fails then the users also suffers.
A lost password is irrecoverable.
Any unauthorized client can hack the client account and can change the data.
1.8 Organisation of the Work
In this project, a secure java chat application is considered which relies on the client-server paradigm to exchange the information. It is divided into five chapters.

Chapter one is the introduction which consists of the background of study, significance of the study, scope of the study, limitations of the study, organization of the work and the definition of terms.
The second chapter focuses on the literature review of relevant scholar’s opinions relevant to this study such as socket programming in java, overview of secure socket layer, hash function e.t.c.
The third chapter gives details of the main methodology and system design to implement the client-server chat application in java. First of all the application is developed by using TCP then and in the end multithreading is used to develop the application. At the end of chapter weaknesses (deadlocks) of multithreading is discussed which can be removed by using synchronizing threads.
Chapter four is the implementation of the secured Java Client-Server Chat Application: it test and analysis the implementation of the application.
Chapter five ends the project report. Firstly, a short summary highlights the main points of the whole project. Next, a number of conclusions and recommendations
are given and lastly Appendix.
1.9 Definition of Terms
Socket:
Socket is a standard connection protocol that supports data communication over the network between connected terminals. The standard connection supports the data transmission both by the TCP and UDP protocols between the terminals.

TCP:
TCP is a transport layer protocol used by applications that require guaranteed delivery of data. Basically, it is a connection-oriented protocol. To communicate over TCP one must first have to establish a connection between pair of sockets, where one socket is client and the other belongs to server. After the connection is established between them then they can communicate with each other.

Client:
A client is a system that accesses or desires for a service made accessible by a server.

Server:
A server is a system (hardware or software) program running to provide the service requests of other system programs.

Port:
Port is a software mechanism that allows the centralized connected Servers to listen for requests made by  clients. Port is actually purposed as a gateway to listen for the requested parameters by the server terminals or other machines. It is a software address on a system that is on the network. Entire request response proceeding among this Application is carries through machine ports.

Network:
This refers to a system were computers are linked to share software, data, hardware and resources for the benefit of users.

Interface:
This may be software or hardware that upon an agreed method spells out the manner a system component can exchange information with another system component.

Secure socket layer (SSL):
This refers to Secure Sockets Layer protocol that is used for encryption of data for secure data transmission.

IP:
This refers to Internet Protocol; it is the reasonable network address of device on a network. It is notational called dotted-decimal (for instance: 128.1.0.1).

Thread:
A thread is a section of code which is executing independently of others threads in a same program. Java has a class Thread which is defined in java.lang package. Thread is the most powerful feature that JAVA supports from other programming languages.

Chapter Five
Summary, Recommendations and Conclusion
5.1 Summary
Client server model is used to communicate over the network where the server is the system that provides services and clients are the systems that want to use these services to communicate with other client systems in the network. In this application, at server side a thread is created that receives numerous clients’ requests. It also contains a list in which Client’s name and IP addresses are stored. After that, it broadcasts the list to all the users who are currently in chat room and when a client logs out then server deletes that particular client from the list, updates the list and then broadcast the list to all available clients. A client firstly must have to register itself by sending username to the server and should have to start the thread so that the system can get the list of all available clients. Then any of two registered clients can communicate with each other.

5.2 Limitations
Some drawbacks of the Client-Server Chat are as follows:

Time Constraint.
Financial Constraint.
In case of server fails then the users also suffers.
A lost password is irrecoverable.
As the server receives as many requests from clients so there is a chance that server can become congested and overloaded.
5.3 Recommendations
Instead of starting a new thread for each task to perform concurrently, the task can be passed to a thread pool. Thread Pools are useful when you need to limit the number of threads running in an application at the same time. There is a performance overhead cost linked with beginning a new thread, because each thread allocates some memory for its stack. This could not be implemented in this application because of time limit. Another suggestion for future works is the use of Java technology newest flavor of TCP Reno. This is because of its light weight and extreme advance features to overcome the flaws of the traditional Transmission Control Protocol.

I recommend that for future works, Thread pool should be used instead of starting new thread for each task. TCP Reno should also be implemented in the future works due to its benefits as mentioned above.

5.4 Bill of Engineering Measurement and Evaluation (BEME)
…this section is available in the complete materials

5.5 Conclusion
A secured chat application has been developed with TCP. TCP is a connectionoriented protocol, so once the connection is established there is no need to send socket address again and again. It is reliable, it guarantees that each packet will arrive and also guarantees that packets will be in the right order.

This project use MySQL for its database to make information in the database secure. The personal details and messages including the private messages in the Database are encrypted using encryptor (one of the security facilities available in the MySQL. As mentioned earlier, MySQL became the obvious choice for the implementation of Secured Java Client-Server Chat Application for the reasons that: MySQL provide secure connections between MySQL clients and the server using the Secure Sockets Layer (SSL) protocol (for the database) to provide secure data communication.

The project implements hash function with the password before the encryption and then stored in the Database;
This application also uses random generated numbers (salt) that is calculated together with the password hash values and stored in the Database. As a result, even if the database is compromised, the salt added to hash values makes it harder to compute the original password. When a random salt is used with the hash function, it significantly increases the strength of encrypting passwords thus makes cracking greatly more difficult. Therefore, makes the chat application server reliable and more secured.

Real network connection is another achievement of this application implementation.

Another accomplishment of this application is private chatting. This is where two users can chat in private. The messages between the users are not displayed / seen in the general chat display textfield. The messages are displayed only within the private message display textfield. It shows and keeps history of the private messages.
As mentioned earlier, this application has been developed placing security of the network as priority, thus, look in details all aspect of network programming lapses, especially client-server chatting system.
########################################################




ABSTRACT

Instant messaging has brought an effective and efficient real-time, text-based communication to the Internet community. In addition, most instant messaging applications provide extra functions such as file transfer, contact lists, and the ability to have simultaneous conversations, which strengthens the reliance of wider sectors of users on these applications. In this project we explore the various attempts to create a unified standard for instant messaging in conjunction with a language translator designed specifically for the three most common languages in Nigeria (Hausa, Yoruba and Igbo). We show the efforts of organizations such as the Internet Engineering Task Force (IETF) in this regard, in addition to some proprietary solutions. We also shed some light on the different types of protocols that are used to implement instant messaging applications. Furthermore, the practical uses of instant messaging are highlighted alongside the benefits that will be reaped by organizations adopting the technology. We dedicate some parts of this project to review current and future research in the field. Various research trends and directions are discussed to show the impact of instant messaging on users, businesses and the decision making process.

 

 

 

 
https://nairaproject.com/projects/3835.html
 

 

CHAPTER ONE

INTRODUCTION

The origin of the Internet begins with the invention and discovery of digital computers in the 1950s. Initial phenomenon of packet networking originated in several computer science laboratories in the United States, United Kingdom, and France. (Kim, Byung-Keun 2005) The US Department of Defence awarded contracts as early as the 1960s for packet network systems, including the development of the ARPANET. The first message was sent over the ARPANET from computer science Professor Leonard Kleinrock's laboratory at University of California, Los Angeles (UCLA) to the second network node at Stanford Research Institute (SRI).

Packet switching networks such as ARPANET, NPL network, CYCLADES, Merit Network, Tymnet, and Telenet, were developed in the late 1960s and early 1970s using a variety of communications protocols. Donald Davies first designed a packet-switched network at the National Physics Laboratory in the UK, which became a testbed for UK research for almost two decades. (Couldry, Nick 2012) The ARPANET project led to the development of protocols for internetworking, in which multiple separate networks could be joined into a network of networks.

 

Access to the ARPANET was expanded in 1981 when the National Science Foundation (NSF) funded the Computer Science Network (CSNET). In 1982, the Internet protocol suite (TCP/IP) was introduced as the standard networking protocol on the ARPANET. In the early 1980s the NSF funded the establishment for national supercomputing centers at several universities, and provided interconnectivity in 1986 with the NSFNET project, which also created network access to the supercomputer sites in the United States from research and education organizations. Commercial Internet service providers (ISPs) began to emerge in the very late 1980s. The ARPANET was decommissioned in 1990. Limited private connections to parts of the Internet by officially commercial entities emerged in several American cities by late 1989 and 1990, (Baran, Paul 1991) and the NSFNET was decommissioned in 1995, removing the last restrictions on the use of the Internet to carry commercial traffic.

In the 1980s, research at CERN in Switzerland by British computer scientist Tim Berners-Lee resulted in the World Wide Web, linking hypertext documents into an information system, accessible from any node on the network. Since the mid-1990s, the Internet has had a revolutionary impact on culture, commerce, and technology, including the rise of near-instant communication by electronic mail, instant messaging, voice over Internet Protocol (VoIP) telephone calls, two-way interactive video calls, and the World Wide Web with its discussion forums, blogs, social networking, and online shopping sites. The research and education community continues to develop and use advanced networks such as NSF's very high speed Backbone Network Service (vBNS), Internet2, and National LambdaRail. Increasing amounts of data are transmitted at higher and higher speeds over fiber optic networks operating at 1-Gbit/s, 10-Gbit/s, or more. The Internet's takeover of the global communication landscape was almost instant in historical terms: it only communicated 1% of the information flowing through two-way telecommunications networks in the year 1993, already 51% by 2000, and more than 97% of the telecommunicated information by 2007. Today the Internet continues to grow, driven by ever greater amounts of online information, commerce, entertainment, and social networking.

Online chat may refer to any kind of communication over the Internet that offers a real-time transmission of text messages from sender to receiver. Chat messages are generally short in order to enable other participants to respond quickly. Thereby, a feeling similar to a spoken conversation is created, which distinguishes chatting from other text-based online communication forms such as Internet forums and email. Online chat may address point-to-point communications as well as multicast communications from one sender to many receivers and voice and video chat, or may be a feature of a web conferencing service.

 

Online chat in a less stringent definition may be primarily any direct text-based or video-based (webcams), one-on-one chat or one-to-many group chat (formally also known as synchronous conferencing), using tools such as instant messengers, Internet Relay Chat (IRC), talkers and possibly MUDs. The expression online chat comes from the word chat which means "informal conversation". Online chat includes web-based applications that allow communication – often directly addressed, but anonymous between users in a multi-user environment. Web conferencing is a more specific online service that is often sold as a service, hosted on a web server controlled by the vendor.

The first online chat system was called Talkomatic, created by Doug Brown and David R. Woolley in 1973 on the PLATO System at the University of Illinois. It offered several channels, each of which could accommodate up to five people, with messages appearing on all users' screens character-by-character as they were typed. Talkomatic was very popular among PLATO users into the mid-1980s. In 2014, Brown and Woolley released a web-based version of Talkomatic.

The first online system to use the actual command "chat" was created for The Source in 1979 by Tom Walker and Fritz Thane of Dialcom, Inc.

 

The first transatlantic Internet chat took place between Oulu, Finland and Corvallis, Oregon in February 1989. (http://securitydigest.org/tcp-ip/archive/1989/02). The numerous limitations of the internet chatting gave rise to instant messaging.

Instant messaging (IM) is a type of online chat that offers real-time text transmission over the Internet. A LAN messenger operates in a similar way over a local area network. Short messages are typically transmitted bi-directionally between two parties, when each user chooses to complete a thought and select "send". Some IM applications can use push technology to provide real-time text, which transmits messages character by character, as they are composed. More advanced instant messaging can add file transfer, clickable hyperlinks, Voice over IP, or video chat.

Non-IM types of chat include multicast transmission, usually referred to as "chat rooms", where participants might be anonymous or might be previously known to each other (for example collaborators on a project that is using chat to facilitate communication). Instant messaging systems tend to facilitate connections between specified known users (often using a contact list also known as a "buddy list" or "friend list"). Depending on the IM protocol, the technical architecture can be peer-to-peer (direct point-to-point transmission) or client-server (an Instant message service center retransmits messages from the sender to the communication device). One the problems faced by Instant messaging since its inception is its inability to instantly translate one language to another so that users speaking different language can easily communicate. It is on this background however that this research work was embarked on to design a multilingual chat application.

1.2.      STATEMENT OF PROBLEM:

In recent years, along with the development of Internet communication technologies, various network-related applications are springing up. In the Web2.0 era, social network and its related applications are the hottest topics. Among them, the instant messaging (IM) has become nowadays an important medium for people to communicate for its convenience and free of use. The instant messaging has shortened the geographical distance between people all over the world - the conversation is as easy as sitting in front of the computer and popping fingers to type – the text communication has become easy and efficient. However, the invisible distance – the barrier results from the different native-languages people speak has not been eliminated yet. Imagine that if there is a Hausa Internet users sitting in front of his computer, how to communicate through the instant messaging to have a conversation with him? So the language barrier during instant messaging is the reason behind this project.

1.3       AIM and OBJECTIVES

The following forms the objective of the study;

    To design a multilingual chat application
    To incorporate all the languages in Nigeria into the chat application so that anybody who speaks any language of Nigeria origin can use it
    To evaluate the existing literature of instant messaging and access the already made multilingual chat application and improve on their deficiencies.

1.5. SCOPE OF THE RESEARCH

This research work is limited to the development of a web based multilingual chat application. The implemented languages are; English, Igbo, Hausa, and Yoruba

1.7. SIGNIFICANCE OF THE STUDY

Hundreds of millions of people use IM to stay connected. In many ways, IM epitomizes the notion of the always-connected, multitasking student, sending and receiving messages at all hours, from a wide spectrum of devices, while doing several other things at the same time. While this dynamic expands access to students who are uneasy with other types of communication, it also provides new modes of expression for students who are otherwise comfortable participating in class.

IM creates an environment that approximates the sharing of a physical space, allowing distance students to engage in learning that approaches face-to-face meetings. The technology is also promoting the practice of creating “back channels,” or secondary conversations that happen at the same time, for example, as a lecture, board meeting, or conference call. Students in a lecture hall might use IM to ask each other questions about the lecture topic and share their thoughts without interrupting the professor. Eliminates use of additional multi-language tools in chat activities and users can retain mono-lingual typical chatting style, even in cross-lingual situations with chat normalization.

1.7 DEFINITION OF TERMS

CERN:

The name CERN is derived from the acronym for the French "Conseil Européen pour la Recherche Nucléaire", or European Council for Nuclear Research, a provisional body founded in 1952 with the mandate of establishing a world-class fundamental physics research organization in Europe. At that time, pure physics research concentrated on understanding the inside of the atom, hence the word "nuclear".

Instant Messaging:

Instant messaging (IM) is a type of online chat that offers real-time text transmission over the Internet. A LAN messenger operates in a similar way over a local area network. Short messages are typically transmitted bi-directionally between two parties, when each user chooses to complete a thought and select "send". Some IM applications can use push technology to provide real-time text, which transmits messages character by character, as they are composed. More advanced instant messaging can add file transfer, clickable hyperlinks, Voice over IP, or video chat.

World Wide Web:

The World Wide Web (abbreviated WWW or the Web) is an information space where documents and other web resources are identified by Uniform Resource Locators (URLs), interlinked by hypertext links, and can be accessed via the Internet. English scientist Tim Berners-Lee invented the World Wide Web in 1989.

Protocol:

This can be defined as a set of rules and regulations that determine how data is transmitted in telecommunications and computer networking. Cryptographic protocol, a protocol for encrypting messages

URL:

A Uniform Resource Locator (URL), colloquially termed a web address, is a reference to a web resource that specifies its location on a computer network and a mechanism for retrieving it. A URL is a specific type of Uniform Resource Identifier (URI), although many people use the two terms interchangeably. A URL implies the means to access an indicated resource and is denoted by a protocol or an access mechanism, which is not true of every URI. Thus http://www.example.com is a URL, while www.example.com is not. URLs occur most commonly to reference web pages (http), but are also used for file transfer (ftp), email (mailto), database access (JDBC), and many other applications.

    Search

search engine by freefind 	advanced
Paper Information

        Format: Ms Word Document
        Pages:   56
        Price: N 5,000
        Chapters: 1-5

        Download This Paper
    Featured Papers
        Making Awesome Presentations: Tips and Tricks for Project Defense
        Developing Outstanding Research Topics
        Challenges of Personal Income Taxation in Ghana
        Work-Life Balance and its Effect on Employee Productivity.
        Effect of Employee Benefits on Organizational Performance
        Impact of Monetary Policies on Foreign Trade in Nigeria

