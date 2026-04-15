ELCTRONIC SIWES RECORD SYSTEM
A CASE STUDY OF THE DEPARTMRNT OF MICROBIOLOGY UMARU MUSA
YAR’ADUA UNIVERSITY KATSINA. (UMYU)
BY
ADAMU SADE
U1/16/CSC/1916
A PROJECT SUBMITTED TO THE DEPARTMENT OF COMPUTER SCIENCE,
FACULTY OF NATURAL & APPLIED SCIENCE, UMARU MUSA YARADUA
UNIVERSITY KATSINA, KATSINA, NIGERIA
IN PARTIAL FULFILLMENT OF THE REQUIREMENTS FOR THE AWARD OF THE
BACHELOR OF SCIENCE (HONOURS) DEGREE IN COMPUTER SCIENCE
SEPTEMBER 2021
DECLARATION
I, ADAMU SADE, hereby declare that this project titled ELECTRONIC SIWES RECORD
SYSTEM has been carried out by me under the supervision of MALAMA AMINA NURA. It
has not been presented for award of any degree in any institution. All sources of information are
specifically acknowledged by means of reference.
……………………………………….. ……………………….
Signature Date
CERTIFICATION
This is to certify that this project “ELECTRONIC SIWES RECORD SYSTEM” has been
carried out by ADAMU SADEof the Department of Microbiology, Faculty of Natural and
Applied Science, Umaru Musa Yar’adua University, Katsina. In partial fulfillment of the
requirements for the award of Bachelor of Science Degree in Computer Science.
________________________________ ________________
Adamu Sade Sign & Date
U1/16/CSC/1916
APPROVAL
This project entitled “ELECTRONIC SIWES RECORD SYSTEM” by ADAMU SADE meets
the requirements governing the award of the degree of Bachelor of Science in COMPUTER
SCIENCE and is approved for its contribution to knowledge and literary presentation.
-------------------------------------------------- -------------------------
Malama Amina Nura Date
Supervisor
-------------------------------------------------- -------------------------
Dr Bashir Isah Dodo Date
Project coordinator
-------------------------------------------------- ------------------------
Dr.AbubakarAminuMu’azu Date
Head of Department
--------------------------------------------------- ------------------------
External examiner
Date
DEDICATION
I will love to dedicate this project work to God for the gift of life and sound health, and to
my beloved parents Alhaji Sade Muhammad (DAN KADAN DAURA) and Fatima Ibrahim
for the great unreserved support they have given me through the course of my study.
ACKNOWLEDGEMENTS
I will like to thank God first for everything that he has done and how He has seen me through
during the course of my study. I will like to appreciate my supervisor Malama Amina Nura for
her intelligent contributions and considerations during the project. It was a great time learning
and working with her and to all the lectures in the department of Computer Science for their
efforts during the years and impacting knowledge to me.
My overflowing gratitude goes to my parent Alhaji Sade Muhammad and to my entire family for
the support and care received during the course of my studies. I will also like appreciate
everyone who has in one way or the other contributed in making this a reality. Thank you for
being there for me. I love you. Big thanks to all my friends and course mates. Working with
them was fun and full of great contributions and ideas.
TABLE OF CONTENTS
Declaration………….
Certification
Approval
Dedication
Acknowledgement
Table of Content
Abstract……………………………….…………………………………………….…...
CHAPTER ONE..........................................................................................................................................1
1.0 INTRODUCTION.................................................................................................................................1
1.1 BACKGROUND OF THE STUDY............................................................................................................1
1.2 STATEMENT OF THE PROBLEM..........................................................................................................3
1.3 AIM AND OBJECTIVES........................................................................................................................3
1.4 SCOPE AND LIMITATION....................................................................................................................4
1.6 CHAPTER SUMMARY..........................................................................................................................4
CHAPTER TWO.........................................................................................................................................5
LITERATURE REVIEW............................................................................................................................5
2.0 INTRODUCTION.................................................................................................................................5
2.1 Discussion on the Study Area............................................................................................................5
2.1.1 Academic Programmes...................................................................................................................5
2.2 OVERVIEW OF ONLINE SIWES REPORT MANAGEMENT SYSTEM.......................................................6
2.3 Review of Existing Relevant Literatures.............................................................................................8
2.3.1 Gap Analysis..................................................................................................................................10
2.3.2 Gap Resaerch................................................................................................................................11
2.4 Summary..........................................................................................................................................12
CHAPTER THREE........................................................................................................................................13
SYSTEM ANALYSIS AND METHODOLOGY.................................................................................................13
3.0 Introduction.....................................................................................................................................13
3.1 System Analysis...............................................................................................................................13
3.1.1 Analysis of the existing system.....................................................................................................13
3.1.2 Analysis of the proposed system..................................................................................................13
3.2 System Requirement For The Proposed System..............................................................................14
3.2.2 Non-Functional Requirements......................................................................................................16
3.3 Methodology...................................................................................................................................16
3.3.1 Interview Method.........................................................................................................................16
3.3.2 Purpose of Interview....................................................................................................................16
3.3.3 Documentary Review Method......................................................................................................17
3.4 System Design..................................................................................................................................17
3.4.1 Input Design...........................................................................................................................17
3.4.2 Output Design........................................................................................................................18
3.4.3 Justification for the waterfall methodology..................................................................................19
3.4.4 Steps Of Design Process................................................................................................................21
3.4.5 System Flow Design......................................................................................................................22
3.5 Chapter Summary............................................................................................................................24
CHAPTER FOUR...............................................................................................................................25
SYSTEM DESIGN AND IMPLEMENTATION..............................................................................25
4.0 Introduction.....................................................................................................................................25
4.1 Objectives Of The Design.................................................................................................................25
4.2 Architectural Design....................................................................................................................26
4.2.1 Webapp Architecture...............................................................................................................26
4.3 System Design..............................................................................................................................27
4.3.1 Physical Design.........................................................................................................................28
4.3.2 Use Case Diagram.....................................................................................................................29
4.3.3 Entity Relationship Diagram......................................................................................................30
4.3.4 Application Algorithm...............................................................................................................31
4.4 Features Of The System...................................................................................................................32
4.4.1 Login Page.....................................................................................................................................32
4.4.2 Admin dashboard.........................................................................................................................33
4.4.3 Admin Manage Student/Staff.......................................................................................................34
4.4.4 Admin Supervisor Allocation Page................................................................................................35
4.5 System Testing And Deployment.....................................................................................................35
4.5.1 Testing Method............................................................................................................................36
4.5.2 Test Plan.......................................................................................................................................37
4.5.3 Testing Procedure.........................................................................................................................37
4.6 Performance Evaluation..................................................................................................................39
4.5.1 Changeover Procedures................................................................................................................40
Advantages................................................................................................................................................41
Disadvantage.............................................................................................................................................41
Recommendation.................................................................................................................................41
4.6 Summary Of The Chapter................................................................................................................41
CHAPTER FIVE............................................................................................................................................43
SUMMARY, CONCLUSION AND RECOMMENDATIONS..............................................................................43
5.0 Introduction.....................................................................................................................................43
5.2 Summary..........................................................................................................................................43
5.3 Conclusion.......................................................................................................................................44
5.4 Recommendations...........................................................................................................................44
5.5 Problem Encountered......................................................................................................................44
5.6further Research...............................................................................................................................45
REFERENCE............................................................................................................................................46
ABSTRACT
The ECLECTRONIC SIWES RECORD SYSTEM is a project carried out due to the need to
improve the working system of the department of microbiology UMYU from the slow unreliable
manual means of record keeping. The developed system enables online monitoring and the
SIWES of student to be stored, It provides an avenue to manage SIWES records and a website
that people can access to get information The system is developed in such a way that it is easy to
access and manage.
CHAPTER ONE
1.0 INTRODUCTION
In this chapter I will present the overview of the whole project, the chapter begin with the
background of the study, the problem that lead to the research, aim and object of the research,
scope of the study and significance of the projects.
1.1 BACKGROUND OF THE STUDY
The Students Industrial Work Experience Scheme (SIWES) is a skills training programme
designed to expose and prepare students of universities and other tertiary institutions for the
Industrial Work situation they are likely to meet after graduation. It is also a planned and
structured programme based on stated and specific career objectives which are geared towards
developing the occupational competencies of participants (Mafe, 2019). Consequently, the
SIWES programme is a compulsory graduation requirement for all Nigerian university students
offering certain courses. The Students Industrial Work Experience Scheme (SIWES), is the
accepted training programme, which is part of the approved Minimum Academic Standard in the
various degree programmes for all Nigerian Universities. The scheme is aimed at bridging the
existing gap between theory and practice of Sciences, Agriculture, Medical Sciences (including
Nursing), Engineering and Technology, Management, and Information and Communication
Technology and other professional educational programmes in the Nigerian tertiary institutions.
It is aimed at exposing students to machines and equipment, professional work methods and
ways of safeguarding the work areas and workers in industries, offices, laboratories, hospitals
and other organizations. Prior to establishing the Scheme, industrialists and other employers of
labor felt concerned that graduates of Nigeria Universities were deficient in practical background
studies preparatory for employment in Industries and other organizations. The employers thus
concluded that the theoretical education being received in our higher institutions was not
responsive to the needs of the employers of labour. Consequently, the rationale for initiating and
designing the scheme by the Industrial Training Funds ITF (Hilton, 2018).
Doug miles (2009) define; Record keeping system is the systematic procedure by which the
records of an organization are created, captured, maintained and disposed of Record management
system is the practice of identifying, classifying, archiving and destroying records. The
International Standards Organization (ISO) (2001) defines record as “The field of management
responsible for the efficient and systematic control of the creation, receipt, maintenance,
evidence of information about business activities and transactions in the form of records”. The
International Council on Archives (ICA) committee on Electronic Records (2014) defined a
record as,” recorded information produced or received in the initiation, conduct or completion of
an institutional or individual activity”. While the definition of a record is often identified
strongly with document, a record can be a tangible object or digital information which has value
to an organization. Records are to be managed according to their value to the organization other
than their physical or logical characteristics. A record is something that represents proof of
existence and that can be used to recreate or prove state of existence, regardless of medium or
characteristics. Records can be either tangible objects, such as paper documents like birth
certificates, driver's licenses, and physical medical x-rays, or digital information, such as
electronic office documents, data in application databases, web site content and electronic mail
(email). Linda shave (2015) said in “The changing landscape Record management in the 21st
century” The digital revolution and the rate that new technology is rapidly evolving is impacting
our workplace. Cloud computing, big data, machine to machine, mobile devices such as tablets,
smart-phones, wearable devices, and social networks are disrupting our traditional ways of
working. A new world is unfolding for the next generation record and information management
professional. The 21st century records and information professional must adapt, they will need to
transition from the analogue world and go on a journey of discovery exploring evolving trends,
challenges and opportunities of the digital era. Sacramento February (2002) in California;
Record Management (RM), also known as Record Information Management or (RIM), is the
professional practice or discipline of controlling and governing what are considered to be the
most important records of an organization throughout the records life-cycle, which includes from
the time such records are conceived through to their eventual disposal. This work includes
identifying, classifying, prioritizing, storing, securing, archiving, preserving, retrieving, tracking
and destroying of records. Roberto Nahum march (2012) a good record keeping is an important
part of accountability for an organization to those who use their service. Record keeping enable
organization to keep a summary of decision made and their reasons for keeping such record is
that they are essential source of evidence for reviews and investigation. National Archives of
Malaysia (2011) defined electronic record management system provide the technological
component of a frame work for the systematic and structured management of record; they link
electronic and non-electronic record to business activities, retain record of past action, and fix the
content and structure of record overtime. This will ensure that the electronic record (e-record)
generated can be preserved while maintaining its authenticity, reliability, integrity, usability, and
accessibility at any time (Nahum, 2012).
1.2 STATEMENT OF THE PROBLEM
The existing manual form of storing and keeping SIWES report book for future use at
department of Microbiology, faculty of natural and applied science, Umaru Musa Yar’adua
University Katsina. Katsina state. That is to say that the process of daily task and activities are
done manually. The problem encounter includes:
• Time wasted and resultant long quarries.
• Loss of report book.
• As a result of this problem the manual system of siwes record system information
storage, input and retrieval is very clumsy.
1.3 AIM AND OBJECTIVES
This project is aimed to develop a web base siwes record system for the department of
Microbiology UMYU katsina.
The specific objectives are: -
• To develop a system that will allow students to submit/update their logbook at
any time via a paperless, environmentally-friendly method.
• To develop a system that will help the department in monitoring their Siwes
students, irrespective of the distance where the student is carrying out his/her
training.
• To develop will help to bridge the communication gap between students and
supervisors during the students IT/SIWES training programme.
1.4 SCOPE AND LIMITATION
The extent to be covered in this project work is, this work creates a database for computerized
siwes record system for the department of Microbiology UMYU Katsina. This project will make
siwes record easier by the student and lecturers respectively, with the system on the network the
lecturers on the department can have access to the document and will help them to know the
number of students that done their report in the departments.
1.5 SIGNIFICANCE OF THE PROJECT
It is hoped that the result of this study will serve as a tool for helping the student to upload their
siwes report and the teachers also to store electronically. Organization large and small are
considering the transition from traditional ‘analogue’ for creating, capturing, storing, managing
and preserving their business information to cloud based solutions, the hyper connected word of
‘digital’ and the internet of things. Interconnection between people, mobile technology, anytime,
anywhere and any place. Digital record keeping system is record systems that allow simplified
record keeping some significant/advantages are: -
• Require less physical storage space than manual or paper based system
• Automatically tallies amount and ease of generating report
• Case of fire or theft
• Capture and access records on the go from different devices
1.6 CHAPTER SUMMARY
In this chapter, it provides an overview of the whole project idea, problem statement, aim and
objective of the project, scope of the project and finally the significant of the project were
mentioned.
CHAPTER TWO
LITERATURE REVIEW
2.0 INTRODUCTION
This chapter presents the theoretical review of some works related to this work. The literature
review is one of the least understood parts of a research project and it’s a summary of previous
research on the topic. Literature reviews can be bibliographic essay that is published separately
in a scholarly journal.
The review will help in designing the methodology and will also enable other researchers to
interpret the research being done.
2.1 Discussion on the Study Area
Umaru Musa Yar’adua University, Katsina commenced academic activities in the 2006/2007
academic session with an initial enrolment of 1,225 students across the 15 different programmes
domiciled at the 3 pioneer Faculties of Natural and Applied Sciences, Humanities and Education.
In 2012/2013 academic session, the University introduced 2 new faculties of Law and Social and
Management Sciences and, in addition to 9 new academic programmes. These increases resulted
in the surge on student’s enrolment of the University. Also in the same year, the University
commenced Postgraduate studies in 14 different programmes for the award of PGDE, M.A.,
M.Ed., M.Sc and PhD.
At the commencement of the academic activities in 2006/07, the University has 3 Faculties, 13
Departments and 26 programmes. In 2012/13 academic session, 9 more undergraduate and 13
postgraduate programmes were established. Between 2013/14 to 2015/16 academic session,
additional 10 postgraduate programmes were established. Presently, there are 5 Faculties, 21
Departments, 35 Undergraduate and 23 Postgraduate programmes in the University.
2.1.1 Academic Programmes
In 2009, the National Universities Commission visited the University for Programme
Accreditation Exercise. During the visit 26 programmes were presented for accreditation in
which 16 programmes earned Full accreditation Status, while the remaining 10 earned Interim
accreditation Status. In 2011, the NUC re-visited the 10 programmes with interim status for re-
accreditation exercise, at the end of the visit; the 10 programmes earned Full accreditation Status
of the NUC. In 2015 the NUC re-visited the University for another round of programme
accreditation exercise for the 16 programmes with Full accreditation status during the 2009
exercise. Also included for this exercise were the 9 new programmes that were established in
2012/13 session. In total, the University presented 25 programmes during the period under
review. The outcome of the NUC visit, all the 25 programmes of the University earned Full
accreditation status. Another programmes re-accreditation exercise was conducted by the NUC
in 2016 for the 10 programmes with Full accreditation Status as at 2011. The result of the
exercise reveals that all the 10 programmes presented earned Full accreditation status.
Presently, all the 35 Undergraduate programmes run by the University have Full accreditation
status.
2.2 OVERVIEW OF ONLINE SIWES REPORT MANAGEMENT SYSTEM
The Students Industrial Work Experience Scheme (SIWES) is a skills training programme
designed to expose and prepare students of universities and other tertiary institutions for the
Industrial Work situation they are likely to meet after graduation. It is also a planned and
structured programme based on stated and specific career objectives which are geared towards
developing the occupational competencies of participants (Mafe, 2019). Consequently, the
SIWES programme is a compulsory graduation requirement for all Nigerian university students
offering certain courses. The Students Industrial Work Experience Scheme (SIWES), is the
accepted training programme, which is part of the approved Minimum Academic Standard in the
various degree programmes for all Nigerian Universities. The scheme is aimed at bridging the
existing gap between theory and practice of Sciences, Agriculture, Medical Sciences (including
Nursing), Engineering and Technology, Management, and Information and Communication
Technology and other professional educational programmes in the Nigerian tertiary institutions.
It is aimed at exposing students to machines and equipment, professional work methods and
ways of safeguarding the work areas and workers in industries, offices, laboratories, hospitals
and other organizations. Prior to establishing the Scheme, industrialists and other employers of
labor felt concerned that graduates of Nigeria Universities were deficient in practical background
studies preparatory for employment in Industries and other organizations. The employers thus
concluded that the theoretical education being received in our higher institutions was not
responsive to the needs of the employers of labour. Consequently, the rationale for initiating and
designing the scheme by the Industrial Training Funds ITF.
The scheme is a tripartite programme involving the students, the universities and the employers
of labor. It is funded by the Federal Government and jointly coordinated by the Industrial
Training Fund (ITF) and the National Universities Commission (NUC).
• To provide an avenue for students in the Nigerian universities to acquire industrial skills
and experience during their course of study;
• To prepare students for the work situation they are likely to meet after graduation;
• To expose the students to work methods and techniques in handling equipment and
machinery that may not be available in their universities;
• To allow the transition phase from school to the world of working environment easier and
facilitate students’ contact for later job placements;
• To provide students with an opportunity to apply their theoretical knowledge in real work
situation thereby bridging the gap between theory and practice.
Knowing fully well that Training is a key factor in enhancing the efficiency and expertise of the
workforce and no society can achieve meaningful progress without encouraging its youth to
acquire necessary practical skills that will enable them to harness available resources to meet the
needs of society- an innovative phenomenon in human resources development and training in
Nigerian tertiary institutions by the industrial training fund (ITF). This innovative phenomenon
was called The Student Industrial Work Experience Scheme (SIWES) otherwise referred to as
Industrial Training (IT). The Student Industrial Work Experience Scheme (SIWES) was
initiated in the year 1973 by the Industrial Training Fund (ITF). It is a Tri-partite programme
involving Students, Universities and Industries. It is funded by the federal government of Nigeria
and jointly coordinated by the ITF and the Nigerian Universities Commission (NUC). It is a skill
training programme designed to expose and prepare students of post-secondary schools (tertiary
institutions) to the industrial work situation they are likely to meet after graduation. A mobile-
based SIWES placement survey system can be described in so many ways, but owing to the
perspective of this study, it is a mobile system that can be installed and run on different mobile
platforms (IOS, Android, Windows Phone, etc.) and different mobile devices (smart phones and
tablets) but are written with web technologies. This system (Mobile-based SIWES placement
survey system) would be a hybrid application that run inside a native container, and leverage the
device’s browser engine (but not the browser) to render the HTML and process the JavaScript
locally. This approach is important so as not to make the system platform dependent.
A portal system can be described in different ways depending on differing point of views. To a
user of a portal, it is a web system that provides the functions and features to authenticate and
identify users. It provides an easy, intuitive, personalized and user-customizable web-interface
for facilitating access to information and services that are of primary relevance and interests to
them. However, to the organization that sets up the portal, it is a system that helps the
organization to catalogue or organize collections of different and multiple sources of information
for dissemination to many users according to their specific privileges, needs and interests.
Therefore, the main purpose for setting up a portal is to bring vast information and resources
available from many sources to many users in an effective manner. There have been several
efforts in Nigeria and in other parts of the world to build portal systems that can facilitate
administration and learning in higher institutions.
2.3 Review of Existing Relevant Literatures
From the establishment of ITF and SIWES scheme, the management and supervision of students
on SIWES has been manual. That is, ITF teams, institution supervisors have to travel across the
nation to supervise students on SIWES. Students have to travel to their institutions to submit
their acceptance letters from their employer industry. The challenges and the risks involved have
made this process very stressful and inadequate leading to the inability of the full realization of
the said objective of the scheme. Several researches have been carried out in an attempt to solve
the challenge of coordination and supervision of SIWES Scheme to ensure full realization of the
set objective. Here, related systems are presented and compared with the proposed system.
A BlogSpot designed and hosted by Federal Polytechnic Oko Anambra, Nigeria –
SIWESFEDPOLYOKO, for the purpose of helping students on SIWES scheme to have access to
information regarding the institution while away. They learn of what is required of them in the
institution. That is, this system does not support any form of supervision but only provides
information for students’ consumption so as to act accordingly (FEDPOLYOKO, 2017).
Adetiba Victor, Egunjobi, &Oladije, (2018), an e-SIWES portal was developed for Covenant
University, Nigeria to automate and enhance the processes of SIWES activities such as
registration, dissemination of information, filling of log book, day-to-day activities as well as
supervision and assessment of students on SIWES by lecturers and industry-based supervisors.
The web-based portal implemented online log book and assessment forms used during SIWES
for logging by students and assessment by institution-based supervisor (lecturer); it doesn’t
support notifications broadcast to all students on SIWES.
Also, Babalola, Adeyemo, & B., (2019), a web-based portal was developed for the AfeBabalola
University, Nigeria following the challenges faced by the manual processes involved in the
university when it comes to SIWES. For supervision, assessment and mentoring, lecturers are
required to travel to all the industry where students are trained which makes the process very
tedious and ineffective. Therefore, the system implemented was to solve such problems. A host
of other institutions in Nigeria (UNIOSUN, 2018; UNIZIK, 2017; UNILAG, 2018; UNILORIN,
2019) have also implemented SIWES portals to enable them manage the processes efficient.
These systems, to the best of our knowledge, have not being published on any literatures.
Lynch, K., A. Heinze and E. Scott, (2019) developed Redmine has an update feature whereby an
issue can be “updated” to reflect any problems and findings associating with the specific
assigned task. The essential process for it to work is unpretentious. Each student will be given an
issue (essentially a task) Corresponding to their name by either from the supervisor or a
teammate, with an estimated date of completion. Once a new issue is submitted, all
corresponding parties are able to track this task to determine whether it meets the estimated
completion deadline or not. One of the supervisor’s tasks in FYP is to track each student’s
progress. There have already been some reasonably good systems put in place for this. In the
initial part of the project, each FYP team is required to plan the entire project duration using
Microsoft Project. The plan would include each task such as design, development and testing.
Students are required to create a Gantt chart for it. A Gantt chart is a type of bar chart that
exemplifies a project schedule. It illustrates the start and finish dates of the terminal elements as
well as the summary elements of a project. The intention of the Gantt chart is to help the FYP
team to plan their work accordingly.
Collins, T., S.I. Wooley, N.C et al. (2020) developed Clarizen's online project management
solution offers users instant gratification with all aspects of online project scheduling – planning,
resource load, task updates, scheduling conflicts and milestone progress. This enables project
managers to react quickly and easily to all changes in the system without having to wait for team
members to "save" or "update" their entries and additions. Instantly view scheduling
dependencies and conflicts – any change made to any project will be instantly updated in the
project scheduling view - enabling you to manage these changes and make adjustments as
needed.
Luisa Martinez., M., (2020) develop web-based online management system for undergraduate's
thesis, which is of great practical for improvement of teaching management and quality. The
system uses ASP.Net, SQL Server for its development, including four types of users: system
administrators, teachers, students and auditors. The paper describes the responsibilities of the
four categories of users, workflow, design ideas, and discusses some design methods to enhance
the security of the system. The system has been widely promoted in some schools of Huaibei
Normal University and achieved good results.
Also, Fraile, R., et al., (2020) come up with web-Based Evaluation for Online Courses and
Learning Management System This system focus on the Web-based evaluation framework of
online courses and learning management system (LMS), based on Web-based questionnaires that
are directed at different target groups for the course contents and the design of the LMS as well
as the Web site. The evaluation criteria are described in more detail and are included in Web-
based questionnaires. More over the system provides a collection of coordination pathways and
interfaces to remove the problems of document access. This system was develop using PHP, JSP
and MYSQL. The respondent in the system require 160 students in the Faculty of University of
Malaya.
2.3.1 Gap Analysis
S/NO NAME YEAR CONTRIBUTIONS LIMITATION
1 FEDPOLYOKO 2017 A BlogSpot designed and hosted by Federal
Polytechnic Oko Anambra, Nigeria –
SIWESFEDPOLYOKO, for the purpose of
helping students on SIWES scheme to have
access to information regarding the institution
while away.
The system does not
support any form of
supervision but only
provides information
for students’
consumption so as to
act accordingly.
2 Adetiba Victor 2018 An e-SIWES portal was developed for
Covenant University, Nigeria to automate and
enhance the processes of SIWES activities
such as registration, dissemination of
information, filling of log book, day-to-day
activities as well as supervision and
assessment of students on SIWES by lecturers
and industry-based supervisors.
It doesn’t support
notifications
broadcast to all
students on SIWES.
3 Babalola,
Adeyemo
2019 web-based portal was developed for the
AfeBabalola University, Nigeria following the
challenges faced by the manual processes
involved in the university when it comes to
SIWES. For supervision, assessment and
mentoring, lecturers are required to travel to
all the industry where students are trained
which makes the process very tedious and
ineffective. Therefore, the system
implemented was to solve such problems.
Reports are not easily
formattable.
4 Collins, T. 2020 Developed Clarizen's online project
management solution offers users instant
gratification with all aspects of online project
scheduling – planning, resource load, task
updates, scheduling conflicts and milestone
progress. This enables project managers to
react quickly and easily to all changes in the
system without having to wait for team
members to "save" or "update" their entries
and additions.
Bugs occur
infrequently and the
software is very
complex.
5 Fraile, R., et al., 2020 Developed an online task management
platform called monday.com it’s a
collaborative environment that allows project
members to communicate, create a knowledge
base and share files, images, designs, and
other specifications. Users can effectively
collaborate and track project progress and
recurring tasks across multiple boards.
Tasks and report
were difficult to see
in terms of deadlines
and progress.
Table 2.1 gap analysis
2.3.2 Gap Resaerch
From the previous system, it is much more focus on providing guidelines and final submission.
Based on my observation, monitoring through online communication must implement in the
system. It is can help the process more effective and efficiency. When comparing the proposed
system with others system, functionality of the system should be considered. The first function in
the system are generate report and update problems. This function is quite important because if
the system not provide this function, it can cause problem and the system will become
complicated.
2.4 Summary
This chapter provides discussion on the study area, academic programmes, an overview of online
siwes report management system, and also it encompasses review of existing relevant
literatures, gap analysis as well as gap research.
CHAPTER THREE
SYSTEM ANALYSIS AND METHODOLOGY
3.0 Introduction
System development life cycle (SDLC) is a process of understanding how an information system
(IS) can support business needs by designing a system, building it and delivering to users. Data
collection is thegatheringand measuring information on targeted variables in an established
fashion, which enables one to answer relevant questions and evaluate outcomes (Dennis et al
2009).
3.1 System Analysis
System analysis is a process of collecting data, understanding the process involved, identifying
problems of existing system and recommending feasible suggestions or solutions for the
improving the system. It’s a problem-solving technique that decomposes a system into it and
various component pieces for the purpose of studying how well those component part work and
interact to accomplish their purpose (Okarfor, B. O, 2013).
3.1.1 Analysis of the existing system
The Industrial Training/Students Industrial Work Experience Scheme is usually intended for
students to get valuable work experience relating to their field of study, due to the difficulty in
getting work placement students often find themselves getting jobs in areas far away from their
institutions. Supervisors have to wait till the end of the training scheme to assess the performance
of the students. As a result of this problem, supervisors find it very difficult to monitor the
progress of the student regularly (Olabiyi, O. S., 2019).
3.1.2 Analysis of the proposed system
The project will cover the student and lecturer/supervisor progress report. The automation
involves an online portal where then student and lecturers/supervisors can interact regularly. The
proposed system will help to bridge the communication gap between students and supervisors
during the students IT/SIWES training programming, it seeks to provide a steady communication
stream via the use of online resources, so as to ensure effective monitoring of students,
irrespective of the distance where the student is carrying out his/her training. By using this
system, students can update their logbook at any time via a paperless, environmentally-friendly
method as well as submit their logbook and final report through online. Supervisors can access
the student's logbook at any time; therefore, they can evaluate and grade the student at their own
page. Student can submit their report and get feedback from their supervisor. Supervisor will
assign marks to students on their progress and performance during presentation. After that,
student able to check their result. The proposed system is an automated solution for
microbiology student problem of umarumusayar’adua university katsina. The online progress log
feature is provided for students to keep updating the progress. This progress is dates and timed.
The supervisor can also put feedback or comments on the progress. This can also be used for
online discussion on aspects of the siwes report. The project will cover the student and
lecturer/supervisor progress report. The automation involves an online portal where the student
and lecturers/supervisors can interact regularly. The proposed system will help to bridge the
communication gap between students and supervisors during the students IT/SIWES training
programming, it seeks to provide a steady communication stream via the use of online resources,
so as to ensure effective monitoring of students, irrespective of the distance where the student is
carrying out his/her training. By using this system, students can update their logbook at any time
via a paperless, environmentally-friendly method as well as submit their logbook and final report
through online. Supervisors can access the student's logbook at any time; therefore, they can
evaluate and grade the student at their own page. Student can submit their report and get
feedback from their supervisor. Supervisor will assign marks to students on their progress and
performance during presentation. After that, student able to check their result. Also, the system
would enhance collaborative management and real-time supervision of students on SIWES as
well as allowing students to report their daily activities. It also allows students to submit their
account details for payment of allowances by the ITF.
3.2 System Requirement For The Proposed System
Software requirement elicitation is a fundamental and critical part of the software development
life cycle. It is generally accepted that the quality of software depends on the requirements upon
which software has been developed. The success or failure of a software development effort is
greatly influenced by the quality of the requirements. Therefore, we require a variety of
elicitation techniques beforehand to determine the user or customer needs. Though it is difficult
to gather complete requirements from the users but choosing the best elicitation technique
available in context with the software characteristics might ensure the completeness of
requirements. In their study, Bell et al. observed that, “The requirement for a system does not
arise naturally; instead, they need to be engineered and have continuing review and revision”.
The requirement of the system is determining what functionalities the system should perform
(Sintheya Rahman 2011). There are two types of system requirement, which are functional and
non-functional requirement.
3.2.1 Functional requirement
The functional requirements are directly related to the functionality of the software, it describes
the core functionality of the application. All functionalities in the system are categorized based
on users’ roles. This section includes the data and functional process requirements. Functional
requirement are those services that the proposed system will provide. The following are some of
the functional requirement of the proposed system:
• Log-in form: The form allows the user to input security password that will allow he/she
to get access to the program or system.
• The main menu: Is a form that contain all the menu such as register new student, add
lecturer profile, assignment page, and then see the Report, the section allows the user to
input his/her details. Report is use to view all necessary allocate record, Exit is used to
end the program.
• Student’s registration page: This page is meant for Admin to register student their detail
information, of which on completion of the registration.
• Report viewer: Allow the user to view, search, edit, delete, print and update record to the
database.
3.2.2 Non-Functional Requirements
Non-functional requirements are requirements which specify the criteria that can be used to
judge the operation of a system. The following are some of the non-functional requirement of the
proposed system:
• Ease of use: The general and administrative view/interface should be easy to use and
intuitive.
• Security: The system should prevent unauthorized access to the system with user
authentication.
• Reliability: It should have no downtime and it should be able to handle multiple
concurrent users.
• Performance: It should have a quick response time.
• Scalability: It should have a database that can handle large amounts of data and also be
able to expand in future.
3.3 Methodology
Methodology is the process ofgatheringand measuring information on targeted variables in an
established fashion, which enables one to answer relevant questions and evaluate outcomes
(kassaye, 2007). Primary data about the existing paper-based. Secondary data was used from
online journals, text books, articles and other literature available about billing systems.
3.3.1 Interview Method
Interview and observation had been conducted on 20th April, 2021 at microbiology department
umarumusayar’adua university katsina. The interviewee is the Siwes supervisor of the
department.
3.3.2 Purpose of Interview
• To uncover further problems regarding the siwes supervision system using manual ways.
• To understand the current methods and approach being used in doing daily supervision
and allocation.
• To identify the main features or functionalities to be integrated into the project prototype.
3.3.3 Documentary Review Method
Documentary review method refers to the analysis of documents that contain information about
the phenomenon under study. Payne and Payne (2004) described the documentary review
method as the technique used to categorize, investigate, interpret and identify the limitations of
other methods like findings in previous studies. Data collection involved review of documents to
gather secondary data that was used in the study. Documents that were reviewed included
textbooks, journals and articles. The documentary review checklist was used to collect secondary
data. It constituted the list of items of information that were obtained from documents, records
and other materials. In order to secure measurable data, the items that were included in the
schedule were limited to those that could be uniformly secured from a large number of case
histories or other records (PusatStatistik, 2014).
3.4 System Design
Design of software involves conceiving, planning out and specifying the externally observable
characteristics of the software product. The goal of the design process is to provide blueprint for
implementation, testing and maintenance activities. Various processes are involved in the design
of any particular system, the process later cultivate to form the new system (Gilbert, 2010). The
purpose of the design phase in this work is to specify a particular system that will meet the stated
requirements.
3.4.1 Input Design
A process of converting user originated inputs to a computer-based format. Input design is an
important part of development process since inaccurate input data are the most common cause of
errors in data processing. Erroneous entries can be controlled by input design. It consists of
developing specifications and procedures for entering data into a system and must be in simple
format. The goal of input data design is to make data entry as easy, logical and free from errors
as possible (Ayesha Yaseen 2015).
In input data design, we design the source document that capture the data and then select the
media used to enter them into the computer. Validations are carried out on all the input fields so
as to get data as accurate as possible.
3.4.2 Output Design
Designing computer output should proceed in an organized manner; the right output element is
designed so that users will find the system executed. When we design an output, we must
identify the specific output that is needed to meet the system. The usefulness of the new system
is evaluated on the basis of their output. Once the output requirements are determined, the
system designer can decide what to include in the system and how to structure it so that the
required output can be produced. For the proposed software, it is necessary that the output
reports be compatible in format with the existing reports. The output must be concerned to the
overall performance and the system’s working, as it should. It consists of developing
specifications and procedures for data preparation, those steps necessary to put the inputs and the
desired output, i.e. Maximum user friendly. Proper messages and appropriate directions can
control errors committed by users. The output design is the key to the success of any system. The
output must be concerned to the system’s working, as it should. Output design consists of
displaying specifications and procedures as data presentation. Users are never left with the
confusion as to what is happening without appropriate error and acknowledges message being
received. Even an unknown person can operate the system without knowing anything about the
system (BinteyMahbub, 2010).
Software design and development methods had changed a lot and the progress in recent times is
rapid and also includes various techniques and methods and they are implemented across the
overall software development process. Software development process can be considered as a
complex process as it involve
s lots of steps towards implementation and a standard life cycle steps are followed across this
process. There are different types of software development models and most of them are proved
to be successful and there are some failures even. Across the overall software development life
cycle requirements gathering can be considered as the vital step and this is the phase where most
of the software development models fail (Sanjay, 2013).
As per the opinions of Sanjay (2013) in the software development process, the requirements
gathering phase happened to be typical task for the entity which are to use the software. The
requirements gathering phase involves the different steps in which gathering the client
requirements is the first step, meeting with the client requirements is the second phase and
satisfying the clients requirements is the final step in this process.
This section will cover the details explanation of methodology that was used to make this project
complete and working well. Findings from this research can and should be used to improve upon
this project in upcoming studies. In order to evaluate this project, the methodology is based on
waterfall software Development Methodology, generally the Software Development Life Cycle
is summarized into three major steps in figure 3.1, which are planning, implementing and
analysis.
Figure 3.1 Software Development Life Cycles.
Figure 3.2 Summaries of SDLC Steps
3.4.3 Justification for the waterfall methodology
Waterfall - This is the original, traditional method of software development. It approaches
software development like you would approach building a house, with the view that changes
after the fact are prohibitively expensive. This is a linear method in which there is a big emphasis
Requirement
Analysis Design Coding Testing
Deployment
and
Mentainace
Planning
Data Collection
Hardware and
Software
Requirement
Implimenting
Testing
Implimenting
Project
Analysis
Analysing the
Performance
Verification and
Conclusion
on collecting requirements and designing the software architecture before doing development
and testing. The advantage of this is that the project is well planned, minimizing on mid-project
costs for changing requirements, and that these projects tend to be well documented (G van der
Waldt 2014). This typically results in major version releases with a significant number of new
features every few years.
Framework Type: Linear
Basic Principles:
1. Project is divided into sequential phases, with some overlap and splash back acceptable
between phases.
2. Emphasis is on planning, time schedules, target dates, budgets and implementation of an entire
system at one time.
3. Tight control is maintained over the life of the project through the use of extensive written
documentation, as well as through formal reviews and approval/signoff by the user and
information technology management occurring at the end of most phases before beginning the
next phase.
Strengths:
1. Ideal for supporting less experienced project teams and project managers, or project teams
whose composition fluctuates.
2. The orderly sequence of development steps and strict controls for ensuring the adequacy of
documentation and design reviews helps ensure the quality, reliability, and maintainability of the
developed software.
3. Progress of system development is measurable.
4. Conserves resources.
The disadvantage is that it is very hard to adjust the feature set in the middle of development,
which often happens as problems are uncovered in development or changing business
environments change what is needed in the software. This is such a problem that many
organizations put in a place a "feature freeze" in which they refuse to alter the features to be
included in a given version once software writing begins, and thus needed features get pushed to
later major versions forcing the users of the software to wait years for those features. Anyone
who has worked on a waterfall managed project has experienced the frequent flaps over feature
changes suggested by software developers, management, and clients who often necessitate an
inefficient micromanagement format all of which are arguments against this process (Oladosu ,
2018).
In preliminary design, the features of the new system are specified, cost of implementing these
features and benefit to be derived are estimated while Structural design is a blue print of a
computer system solution to the problem, in structural design the database where input, output
and processing take place are drawn up in detail. (Přibyl, 2011). The whole system was also
designed using flowchart in order to indicate how data flow within the system.
3.4.4 Steps Of Design Process
• Logical design: In the logical design, the inputs, outputs (result), Databases (data stores),
procedures (Data Flow) and boundaries of the system are described, which needs the user
requirement. It specifies the user need at a level of detail that virtually determines the
information flow in and out of the system along with the required data resources.
• Physical design (Database design): This process is concerned with the design of the
physical database. A key is to determine how the access paths are to be implemented. A
physical path is derived from the logical path. The relationships existing among the
entities like one-to-one, one-to-many, many-to-many are considered while designing the
database. Relational structured database is used in this system.
• Program Design In conjunction with database design is a decision on the programming
language to be used and choice of coding. In this design, PHP is used as server-side script
while JavaScript 45 and HTML are used as client-side script, which are supported by
most browsers such as Internet Explorer 5, safari, Mozilla Firefox, Google chrome, and
opera etc.
3.4.5 System Flow Design
Sequential diagram employed in this design is flowchart in order to show the stepwise
procedures used in performing a task within the application. A flowchart consists of special
geometric symbols connected by arrows. Within each symbol is a phrase representing the
activity at that step. The shape of the symbol indicates the type of operation that is to occur. For
instance, the parallelogram denotes input or output. The arrows connecting the symbols, called
flow lines, show the progression in which the steps take place. Flowcharts should “flow” from
the top of the page to the bottom. Although the symbols used in flowcharts are standardized, no
standards exist for the amount of detail required within each symbol. The main advantage of
using a flowchart to plan a task is that it provides a pictorial representation of the task, which
makes the logic easier to follow. The steps and how each step is connected to the next are
showed. This project consists of three sections administrator part, cashier part and consumer’s
section, the way data will flow in each part will be shown in the subsequent flowchart.
NO
YES
NO
YES
START
DISPLAY USER LOGIN
Username
And
Password
DISPLAY MAIN MENU
Error
Fig. 3.3 system data flow
diagram
ENDPRINT REPORT
Fig.3.4 Complete system flow
3.5 Chapter Summary
In this chapter we discussed about system analysis and methodology, analysis of the existing and
proposed system, advantage of the proposed system, system design methodology, system
modelling by the use of use case diagram, class diagram, dataflow diagram, entity relation
diagram and system flowchart.
CHAPTER FOUR
SYSTEM DESIGN AND IMPLEMENTATION
4.0 Introduction
This chapter focuses on how the system was design, implementation and tested a set of
requirement needed to implement the system. Finally, the system is to be tested to ensure that it
is working perfectly and correspond to the design objectives.
Software design commences as the iteration of requirements engineering comes to a conclusion.
The intent of software design is to apply a set of principles, concepts, and practices that lead to
the development of a high-quality system or product. The goal of design is to create a model of
software that will implement all the software requirements correctly and bring delight to those
who use it (pressman, et al, 2015).
The design process moves from a “big picture” view of software to a narrower view that defines
the detail required to implement a system. The process begins by focusing on architecture.
Subsystems are then defined; communication mechanisms among subsystems are established;
components are identified, and a detailed description of each component is developed. In
addition, external, internal, and user interfaces are designed.
4.1 Objectives Of The Design
The objectives of software design are numerous; this include among others as stated by
(pressman, 2015):
i. Design allows us to produce a model of the system or product that is to be built; this
model can be assessed for quality and improved before code is generated, tests are
conducted, and sometimes end users become involved in large numbers.
ii. Design is the place where software quality is established.
iii. Design is the only way that you can accurately translate user’s requirements into a
finished software product or system
iv. Software design serves as the foundation for all the software engineering and software
support activities that follow. Without design, you risk building an unstable system—one
that will fail when small changes are made.
v. One goal of software design is to derive an architectural rendering of a system. This
rendering serves as a framework from which more detailed design activities are
conducted
4.2 Architectural Design
Software architecture alludes to “the overall structure of the software and the ways in which that
structure provides conceptual integrity for a system” (Sha95a). In its simplest form, architecture
is the structure or organization of program components (modules) which comprise software
components, the externally visible properties of those components, and the relationships among
them, the manner in which these components interact, and the structure of data that are used by
the components.
The architectural model of a software is derived from three sources:
i. Information about the application domain for the software to be built.
ii. Specific requirements model elements such as use cases or analysis classes, their
relationships and collaborations for the problem at hand.
iii. The availability of architectural styles and patterns.
4.2.1 Webapp Architecture
Webapp architecture describes an infrastructure that enables a Web-based system or application
to achieve its objectives. Jacyntho and his colleagues [Jac02b] describe the basic characteristics
of this infrastructure in the following manner:
“Applications should be built using layers in which different concerns are taken into account; in
particular, application data should be separated from the page’s contents (navigation nodes) and
these contents, in turn, should be clearly separated from the interface look-and-feel (pages)”
The authors suggest a three-layer design architecture that decouples interface from navigation
and from application behavior. They argue that keeping interface, application, and navigation
separate simplifies implementation and enhances reuse.
In a webapp, “the view is updated by the controller with data from the model based on user
input”. User requests or data are handled by the system. The system also selects the view object
that is applicable based on the user request. Once the type of request is determined, a behavior
request is transmitted to the model, which implements the functionality or retrieves the content
required to accommodate the request. The model object can access data stored in a corporate
database, as part of a local data store, or as a collection of independent files. The data developed
by the model was formatted and organized by the appropriate view object and then transmitted
from the application server back to the client-based browser for display on the user’s platform.
The figure below depicts the architectural design of the web based computer assessment system.
4.3 System Design
Design engineering encompasses a set of principles, concept and practices that lead to the
development of a high quality system or product. Design is also seen in software engineering as
being at the technical kernel of software engineering and is used regardless of software process
model that is use.
The design process moves from a “big picture” view of software to a narrower view that defines
the detail required to implement a system. The process begins by focusing on architecture.
Subsystems are defined; communication mechanisms among subsystems are established;
components are identified, and a detailed description of each component is developed. In
addition, external, internal, and user interfaces are designed.
Client-site (Browser)
ADMIN
APPLICATION SERVER
DATABASE
STUDENT
Client-site (Browser)
The system design was presented using; architectural diagram, use case diagrams, entity-
relationship diagram and data flow diagram
4.3.1 Physical Design
Once the design model is created, you should also conduct reviews of the system design and the
object design. The system design depicts the overall product architecture, the subsystems that
compose the product, and the manner in which subsystems are allocated to processors, the
allocation of classes to subsystems, and the design of the user interface.
The input used in this system is all the information about the student and the exam. The system
accepts inputs from the user (student) via the user interface through the keyboard; the system
also provides user interface from where the student will provide his name/email and password to
login.
The end result of its activities/information generated over a certain period of time is its output.
The output of the system is the actual output based on the questions answered by the student are
ascertained by the system admin.
Below is the physical design of the system and the input/output functionalities.
4.3.2 Use Case Diagram
The purpose of a use case is to define a piece of coherent behavior without revealing the internal
structure of the system. The use cases do not mention any specific algorithm to be used or the
internal data representation, internal structure of the software, etc. A use case typically represents
a sequence of interactions between the user and the system. These interactions consist of one
mainline sequence. The mainline sequence represents the normal interaction between a user and
the system. The mainline sequence is the most occurring sequence of interaction.
Below is the use case for the electronic siwes record system.
STUDENT ADMIN
Upload report
r
LOGIN
4.3.3 Entity Relationship Diagram
The Entity Relationship Diagram (ER diagram) is a semantic data modeling tool that is used to
accomplish the goal of abstractly describing or portraying data. Abstractly described data is
called a conceptual model. Our conceptual model will lead us to a "schema." A schema implies
a permanent, fixed description of the structure of the data. Therefore, when we agree that we
have captured the correct depiction of reality within our conceptual model, our ER diagram, we
can call it a schema. As the name implies, an ER diagram models data as entities and
relationships, and entities have attributes.
View report
View profile
Delete report
LOGOUT
ADMIN
Username
Password
STUDENT
Username
Password
Name
Reg. Number
FILE
Siwes (PDF
format) less
than (2 MB)
4.3.4 Application Algorithm
Fig.3.4 Complete system flow
4.4 Features Of The System
The below figures show some of the features of the system;
4.4.1 Login Page
Fig. 4.1 User login
Figure 4.1 this provides an interface through which users, supervisor, staff and admin can
login respectively. What was required using Mozilla Firefox as the browser or any other
browser, which is available in Microsoft windows operating system. The web directory is log
on to by typing http://localhost/siwesmanagementsystem/index.php in the address bar and
click go on the menu bar or press enter on the keyboard. On connecting successfully to the
host server, the “Home page” is loaded first on the browser. All other pages can be viewed
when user login successfully by clicking their respective links.
4.4.2 Admin dashboard
Fig 4.2 Admin dashboard of the system
The above fig 4.2 is an admin dashboard that allow admin to manage all the activities of the
system such as managing students, manage supervisors, manage siwes posting and admin is also
allow to manage the system users.
4.4.3 Admin Manage Student/Staff
Fig 4.3 Admin manage student/staff
The above fig 4.3 is page that allows admin to manage students/staffs of the system, it enable
admin to add, edit and delete the student or staff.
4.4.4 Admin Supervisor Allocation Page
Fig 4.4 Admin supervisor allocation page
The above fig 4.4 is an admin page that allows admin to allocate the group of student that are
doing their siwes in the same organization to a supervisor so that they can submit their daily
siwes log to him.
4.5 System Testing And Deployment
Software testing is an investigation conducted to provide stakeholders with information about the
quality of the product or service under test. Software testing can also provide an objective,
independent view of the software to allow the business to appreciate and understand the risks of
software implementation. Software testing is the process of validating and verifying that a
project: (CIGNET, 2013).
1. Meets the requirements that guided its design and development.
2. Works as expected.
The testing of the system is done based on two strategies. First, each module or program was
tested independently to ascertain its functionality and the performance of the task defined in its
structure. This process of testing is known as unit test. Since the system is made up of a
collection of different modules (classes), all existing in hierarchy and tied together to actualize
the task of student project allocation system. The next level of testing involves putting all the
modules together and testing them all at once. This interestingly was achieved with the hypertext
reference (href) function in the PHP source module; this method of testing is called integrating
testing. Integration testing is done in terms of interface testing, function calls, input/output
operation as well as storage. To this effect, integration gives a true picture of the system, how it
works and the overall appraisal of the system should be done here by the users. Testing is a
process, which reveals errors in the program. It is the major quality measure employed during
software development. During testing, the program is executed with a set of test cases and the
output of the program for the test cases is evaluated to determine if the program is performing as
it is expected to perform. Test Case design method the developer with a systematic approach to
testing. Method provides a mechanism that can helps to ensure the completeness of tests and
provide the highest likelihood for uncovering errors in software.
Any Engineering product can be tested in one of two ways:
 Knowing the specified functions that a product has been designed to perform, tests can
be conducted that demonstrate each function is fully operational while at the same time
searching for errors in each function. This approach is called Black Box Testing.
 Knowing the internal workings of a product, tests can be conducted to ensure that all
internal operations are performed according to specifications all internal components have been
adequately exercised. This approach is called White box Testing.
4.5.1 Testing Method
One of the strategies used to develop a successful system is to fulfil the system requirement
specification and ensure the system is bug free. In order to ensure the system is developed under
requirement specification and error free, the author has designed an appropriate test method to
test the system. System testing is to examine the system performance, especially in the system
function process. Such as the system input (data) and output (information), this process is to
ensure the input are validated and output are matched with the expected output. In order to
overcome the problem and verify the system, the author has studied various type of testing
method. To have an efficient test on the system, more than one testing method are used to test the
system or a combination of more than one test method.
4.5.2 Test Plan
A test plan has been designed by the author in order to test the system properly. There are three
main categories to be tested, which are the supervisor site, staff site and the Administration site.
The author used unit testing, integrated testing and system testing to test the system. Unit testing
is the first tested on the proposed system. It focuses on each unit of the system and checks for the
source code of the particular unit and checks whether it is operated according to the requirement.
Next, Integrated testing which basically concentrates on the testing of the combined parts of the
system and checks whether all the parts combined together, would work properly. Lastly, system
testing used to test the overall proposed system with all the subsystem integrated together into a
working system.
4.5.3 Testing Procedure
There are lots of testing available, such as unit testing, integration testing, system testing, black
box testing, white box testing, acceptance testing, stress testing, alpha testing, beta testing and so
on. These entire testing types have their own area of applicability. Normally, the testing is used
by the developer to test for the performances, functionality, and reliability of the system. It will
always be easy to find out the error in small area and fix it before going to the large area.
 Unit Testing: Unit testing involves single isolated module. Ideally, a programmer unit
tests each module before trying to integrate that module with other. Testing low-level
modules requires drivers to provide input and output while testing-high level module
requires subs for missing lower-level modules. The entire purpose of unit testing is to
reduce the effort of integration testing. Although the programmer might not unit test all
modules, but they must plan a module test for each module. The plan is simply what data
to provide to the module and what result to expect. This type of testing is sometimes
called black box testing
 Integration Testing: The integration test combines all the modules together and is tested.
First the programmer would add one or two other subordinates from the same level. Once
the program has been tested with the co-ordination module and all of it immediate
subordinate modules, the analyst would add modules from the next level and then test the
program. This procedure would be repeated until all the modules have been tested.
 System Testing: System testing are designed to verify if the finished system meets its
requirement. There are three kinds of system testing. Alpha testing is a system testing
performed within the development organization. Beta testing is the system testing
performed by a selected group of volunteers. Acceptance testing is system testing
performed by user to determine whether to accept the delivery of the system.
Testing is critical for a newly developed system as a prerequisite for rolling out, which is
conducted to ensure accuracy and reliability. Component or unit testing is a method by which
individual units of source code, sets of one or more computer program modules together with
associated control data, usage procedures and operation procedures are tested to determine if
they are fit for use (Huizinga &Kolawa, 2007). In the development of the information
management system, each component was tested independently before finally integrating all of
them into one system. These tests were used to verify that every input of data was assigned to
fields in the appropriate tables. Integration testing is the testing of the interaction among the
system’s components sequentially and continuously. This usually entails the interaction between
the user interface and the database. The interaction between the database and interfaces was
among the integration tests done. The seamless interaction between the User login interface and
the database is illustrated in Figure 25. When another user tries to enter and the user is not
authorized or the user enters incorrect login details, the system displays an error message
“username or password incorrect.”
A system normally consists of all components that makeup the total functioning system. It is
necessary to ensure the entire system runs smoothly and meets desired expectations. Here,
technical, and functional testing were performed. The technical testing involved the process of
testing the systems compatibility with the hardware, operating system, data integrity in the
database, and user authorization access rights. Functional testing was also carried out to establish
how the system would function in its intended working environment. Release testing involves
testing a version of the system that can be released to users. This test is concerned with showing
whether or not the system is working. It is usually black box testing which is based on an
analysis of the expected functionality of the application without reference to its internal
workings.
Our application was tested by the end users to ensure that it met the requirements sufficiently
well.
4.6 Performance Evaluation
Performance testing is designed to test the run-time performance of software within the context
of an integrated system. Performance testing occurs throughout all steps in the testing process.
Even at the unit level, the performance of an individual module may be assessed as tests are
conducted. However, it is not until all system elements are fully integrated that the true
performance of a system can be ascertained.
Evaluation is a systematic determination of a subject’s merit, worth and significance, using
criteria governed by a set of standards. It can assist an organization, program, project or any
other initiative to assess any aim, realizable concept/proposal or any alternative to help in
decision-making (International Center for Alcohol Policies, 2014). The primary goal of
evaluation, in addition to gaining insight into prior or existing initiatives, is to enable reflection
and assist in the identification of future change (The Evaluation Trust, 2007). The DeLone and
McLean Information Systems Success Model (DMSM) for measuring Information Systems (IS)
has a basic model consisting of six categories of Information System success, which are systems
quality, information quality, use, user satisfaction, individual impact and organizational impact
(Palmius, 2007). Concerning systems quality, the proposed system proves effective as it does
what it was required to do with efficiency and speed. Concerning information quality, aside from
the limitation of human error in user input into the system, the system retains quality information
through the removal of redundancies and privacy protection through user authentication before
access to the system. Concerning the use of the system, the proposed system proved easy to use
as all fields are well labeled. Concerning user satisfaction, through the release testing, it was
discerned that the system was satisfying to the users. Concerning individual and organizational
impacts, these aspects could not be properly evaluated due to time constraints to observe and get
information from the users over a long period to know if the system impacted the individuals‟
job performances and organizational improvement through quality information and ease of
information retrieval.
Below is the performance test evaluation of the system:
MODULE DATA PROCESS RESULTS
Homepage The general interface of
the software
First system interface after
login
No errors
Admin login panel Admin email id
Password
Click login button No errors
Staff/supervisor login
panel
Staff/supervisor email id
Password
Click login button No errors
Upload Report Research report Click Upload Report No errors
View Report Research report Click View Report No errors
Delete Report Research report Click Delete Report No errors
4.5.1 Changeover Procedures
The process of putting the new information system online and retiring the old system is known as
system changeover. There are four changeover methods which are:
 Direct cutover: The direct cutover approach causes the changeover from the old system
to the new system to occur immediately when the new system becomes operational. It is
the least expensive but involves more risks than other changeover methods.
 Parallel operation: The parallel operation changeover method requires that both the old
and the new information systems operate fully for a specified period. Data is input to both
systems and output generated by the new system is compared with the equivalent output
from the old system. When users, management, and IT group are satisfied that the new
system operates correctly then the old system is terminated. It is the costliest changeover
method and involves lower risks.
 Pilot operation: The pilot changeover method involves implementing the completely
new system at a selected location of a company. Direct cutover method and operating
both systems for only the pilot site. The group that uses the new system first is called the
pilot site. By restricting the implementation to a pilot site reduces the risk of system
failure as compared with is less expensive than a parallel system.
 Phased operation: The phased operation changeover method involves implementing the
new system in stages, or modules. We can implement each subsystem by using any of the
other three changeover methods. In this approach risk of errors or failures is limited to the
implemented module only as well as it is less expensive than the full parallel operation.
For deploying of the system, we can use any of the above methods but there are some advantages
as well disadvantages of using these systems, which are explained below:
 Pilot operation:
Advantages
Pilot operation is combination of both direct cutover and parallel operation, which
restricts the implementation to a pilot site and reduces risk of system failure as compared
with a direct cutover method.
Operating system only at pilot site is less expensive than parallel operation for entire
organization.
If we use parallel approach to complete the implementation then the changeover period
can be much shorter if system proves successful at the pilot site so a lot of time will be
consumed at hospital in implementing the new system.
Disadvantage
This method is also costly as compared to the direct cutover.
Recommendation
As we can determine from above information that pilot approach is the best approach where we
can see the combination of fewer risks as well as less implementation cost.
4.6 Summary Of The Chapter
This chapter provide details about system design and system implementation. The system design
is concerned with the process and procedures used in transforming the present manual system
into a computerized online system. While the system implementation putting the newly designed
system into existing (operation). This online software is design to handle user’s request. And also
to accept data’s of different user’s concurrently. Moreover the system testing is carried out to
ensure that the system is working properly, and the desire objective is achieved.
CHAPTER FIVE
SUMMARY, CONCLUSION AND RECOMMENDATIONS
5.0 Introduction
This chapter summarizes the study by highlighting the research conducted on the topic. The
conclusions given were drawn from the outcomes of the research and observations on the Siwes
reporting system in department Microbilogyumarumusayar’adua university katsina. Moreover,
recommendations were based on the findings and conclusion of the study.
5.2 Summary
This project hasprovide an online web-based system will help to bridge the communication gap
between students and supervisors during the students IT/SIWES training programming, it seeks
to provide a steady communication stream via the use of online resources, so as to ensure
effective monitoring of students, irrespective of the distance where the student is carrying out
his/her training. The main aim of project is to replace the existing manual method of siwes
reporting system in microbiology department, Umyuk. To effectively drive home the meaning
and concept of this system, the project is subdivided into five chapters, the first chapters
evaluates the background of the study, stating the problems that the newly proposed system is
about to resolve. In furtherance it explicated the significance, and defines the terms and
terminologies required for the implementation and use of this system. The second chapter of this
project reviewed relevant literatures that reiterated on the necessity of the implementation of
Siwes reporting system. The third chapter of this project evaluated the system analysis and
design. It laid emphasis on the nature of the manual systems and described the need for the new
system and its advantages over the old system. The fourth chapter is based on the system
implementation and how to use the system for effective performance. The final chapter
summarised and draw conclusion based on the functionality of the system and make possible
recommendation on how to enhance the system.
5.3 Conclusion
In conclusion this project will cover the student and lecturer/supervisor progress report. The
automation involves an online portal where then student and lecturers/supervisors can interact
regularly. The system will help to bridge the communication gap between students and
supervisors during the students IT/SIWES training programming, it seeks to provide a steady
communication stream via the use of online resources, so as to ensure effective monitoring of
students, irrespective of the distance where the student is carrying out his/her training. By using
this system, students can update their logbook at any time via a paperless, environmentally-
friendly method as well as submit their logbook and final report through online. Supervisors can
access the student's logbook at any time. Student can submit their report and get feedback from
their supervisor. Supervisor will assign marks to students on their progress and performance
during presentation. Also, the system would enhance collaborative management and real-time
supervision of students on SIWES as well as allowing students to report their daily activities.
5.4 Recommendations
In the development of this academic performance evaluation system, I will recommend that if
there is going to be any modification the new writer should endeavor to improve on the
limitations such as inclusion of the online video, voice and chatting system to further increase the
system architecture and to satisfy users need more for writing of the source code, latest PHP
version should be used and mariaDB for the database. There are some limitations during the
development of this system that will require improvement as stated in previous chapter writer
should put them in mind and face it as a challenge and not a problem.
5.5 Problem Encountered
A lot of challenges surfaced during the development of this incredible application though it tried
stopping this project but the doggedness and consistency of the writer was in match with the
challenge.
The following are some of the problems or challenges encountered.
 Expensive internet facility.
 Inadequacy of power supply and many more.
 Time factor on research to get a way of packaging the application successively.
5.6further Research
In the future, the following components can be added to the system in order to improve the
effectiveness and efficiency of the system, which includes:
1. Online voice, video and chatting system should be included.
2. A good internet backup should be automated after everyday activities.
REFERENCE
i. Abdullahi A. O. 2009 Siwes Report, Covenant University,Ota, Nigeria.
ii. Michele Davis, Jon Phillips. 2006. Learning PHP and MySQL, O’Reilly.
iii. Howard S.2012 All About Web Portal: A Home Page Doth Not a Portal Make. Accessed
online at http://net.educause.edu.
iv. SIWES Portal. 2012 Federal Polytechnic, Oko, Nigeria. Accessed online at
http://siwesfedpolyoko.blogspot.com About HKU Portal. Accessed online at
http://www.itservices.hku.hk/portal/
v. A learning portal management system for training the employees, Management
Information Systems Department, Hisar Campus, BogaziciUniversity,Istanbul,Turkey.
Accessed online at http://www.boun.edu.tr
vi. Donald B. 2003 UML Basics, Part II: Activity Diagram. Accessed online at
http://www.ibm.com/developerworks/rational/library/content/RationalEdge/sep03/
f_umlbasics_db.pdf
vii. Robert C.M.2012 UML Tutorial :Part1—Class Diagram. Accessed online at
http://www.objectmentor.com/resources/articles/umlClassDiagrams.pdf
viii. Jason N. and Marcia P. 2001 Building a Secure Web Server. Distributed Systems
Department, Earnest Orlando Lawrence Berkeley National Laboratory.
ix. Armstrong M (2006). A handbook of human resource management practice (10 ed.),
Kogan Page Publishers.
x. Beer M, Ruh RA (1976). Employee growth performance management. Harv. Bus. Rev.,
July-August, pp 59-66.
xi. Brumbach GB (1988). Some ideas, issues and predictions, about performance
management, Pub. personnel manage., Winter: 387-402.
xii. Carpinetti LC, Gerolamo MC, Galdámez EV (2002). ‘Continuous Innovation and
Performance Management of SME Clusters’, Creativity Innov. Manage., 16(4): 376 385.
xiii. Chau VS (2008). ‘The relationship of strategic performance management to team
strategy, company performance and organizational effectiveness’, Team Perform.
Manage. 14(3/4): 113-117.
xiv. GO¨BR, Mccollin C, Ramalhoto MF (2007). ‘Ordinal Methodology in the Analysis of
Likert Scales’, Quality Quantity, 41: 601– 626.
xv. Ingram H, McDonnell B (1996). ‘Effective performance management the teamwork
approach considered’, Managing Service Quality, 6(6):38– 42 MCB University Press.
xvi. Joseph AG, Rosemary RG (2003). ‘Calculating, Interpreting, and Reporting Cronbach’s
Alpha Reliability Coefficient for Likert-Type Scales’, Midwest Research to Practice
Conference in Adult, Continuing, and Community Education, pp. 82-88, Ohio State
University, Columbus, OH.
xvii. Lam TY (2008). ‘Optimisation of performance management for housing services’, J.
Facil. Manage., 6(3): 226-240.
xviii. Lawler EE, Mohrman SA, Gerald E, Ledford J (1998). Strategies for High Performance
Organizations: The CEO Report, San Francisco, Jossey-Bass.
xix. Lawrie G, Cobbold I, Marshall J (2004). ‘Corporate performance management system in
a devolved UK governmental organisation’, Int. J. Prod. Perform. Manage., 53(4): 353-
370.
xx. Luthans F, Rhee S, Luthans BC, Avey JB (2008). ‘Impact of behavioral performance
management in a Korean application’, Leadersh. Organ.Dev. J., 29(5): 427-443.
xxi. Marr B (2006). Strategic Performance Management, 1st edn, Else vier Ltd.
xxii. Mondy RW, Noe RM, Premeaux SR (2002). Human resource management (8th edn),
Upper Saddle River, NJ, Prentice Hall.
xxiii. Mafe, JI (2016). ‘Performance management model A systems-based approach to public
service quality’, The Int. J. Public Sector Manage., 13(1): 19-37.
xxiv. Reilly P (2003). ‘New Approaches in Reward: Their Relevance to the Public Sector’,
Public Money Manage., pp. 245-252.
xxv. Verbeeten FH (2008). ‘Performance management practices in public sector
organizations Impact on performance’, Accounting, Auditing Accountabil. J., 21(3): 427-
454.
xxvi. Waal AA (2006). ‘The Role of Behavioral Factors and National Cultures in Creating
Effective Performance management systems’, Syst. Practice Action Res., 19(1): 6179.
xxvii. Waal AA (2007). ‘Is performance management applicable in developing countries? The
case of a Tanzanian college’, Int. J. Emerging Mark., 2(1): 69-83.
xxviii. Waal AA, Coevert V (2007). ‘The effect of performance management on the
organizational results of a bank’, Int. J. Prod. Perform. Manage., 56(5/6): 397-416.