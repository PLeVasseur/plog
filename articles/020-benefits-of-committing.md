# The Benefits of Committing

About a year ago a project I had been working on at General Motors for six years was cancelled. This time lead to a lot of soul-searching on what I believed in and wanted to do next. I was able to find something new within GM I could hitch my wagon to -- [Eclipse uProtocol]() -- as a place I could develop Rust software in an open source environment. Ultimately that project too was more-or-less let go of when myself and many others which worked on Eclipse uProtocol were caught up in a restructuring in August of 2024.

Those are stories for another time, however! Today I'd like to go over some of the benefits we can see when fully committing to seeing a product through to delivery over a time span measured in years.

## Why commit?

In the era of the Zero Interest Rate Phenomenon (ZIRP), the run from essentially 2012 till 2021, there was extreme growth felt within the tech sector. Gergly Orosz of The Pragmatic Engineer gave a great [talk](https://blog.pragmaticengineer.com/the-software-engineering-industry-in-2024/) about some of the reasons behind this. But in my opinion one of the unfortunate side effects is a population of senior engineers that have job hopped for most of their careers, which may have some broad base of skills but haven't seen something through to completion.

I'd almost call this group of folks "consultants" rather than saying for example that they were full-time employees of four businesses over eight years. There's nothing wrong with being a consultant per-se. You gain a set of skills for how to quickly adapt to new situations and start bringing value. You're exposed to a variety of different pains which are felt in industry. I'm starting to see this with my Rust consultancy I've started, Oxidation Partners.

On the other hand, I think it was said well by Steve Jobs in his [talk](https://www.youtube.com/watch?v=-c4CNB80SRc) he gave at MIT:

> Without owning something over an extended period of time, where one has the chance to take responsibility for one's recommendations, where one has to see one's recommendations through all action stages and accumulate scar tissue for one's mistakes and pick oneself up off the ground and dust oneself off, one learns a fraction of what one can.

## How to commit?

Commit to some cause that you can believe in. I stuck with the product I worked on for six years through great times and some fairly poor times because I believed in the vision of making safer cars for customers through automated driving features. I thought that what we were doing was revolutionary for the Automotive industry in moving automated driving to a wider operational design domain using some cutting edge technology. We also had a focus on delivering a safe system, one which had safeguards in place and made use of various sensor modalities for robustness. I really appreciated the culture of safety we embraced.

## What does committing look like?

### Embrace the uncomfortable

When truly committing to a project you believe in, there may be things you're asked to do which are outside of your comfort zone. Embrace those chances as you will learn a lot.

I worked as a Technical Project Manager, then Technical Lead, then finally as an Individual Contributor throughout my time on this project, doing what was needed on the project as needed, then yielding to others whom I thought would be a better fit.

I had no clue what I was doing as a TPM at first. I had always worked as a part of smaller teams in research and advanced engineering organizations where the mode of operation was: "we're just going to do stuff and then show customers". I learned a lot about the dos and donts of technical staff management and got some bruises along the way. These experiences helped me understand how to scale our team working on Rust software at General Motors developing Eclipse uProtocol. When to lay back and let the team tackle a problem; what the signs were that I needed to provide direction; how to develop roadmaps, get buy-in from leadership, and staff up a team.

When I then moved into a Technical Lead role I again had never truly worked on applied research to that level. I was thankful to work with an engineer from GM's R&D side to build out a new algorithm to improve the performance of an association system between perceived and mapped lane markings. We brought the work far enough to be worthy of a [patent](https://patents.justia.com/patent/20230050706). The experience of bringing something from zero to one was great as I then had the opportunity to work with algorithms engineers to implement the math. I made use of this experience during my work on the Eclipse uProtocol [uStreamer](https://github.com/eclipse-uprotocol/up-streamer-rust) as I worked with the project lead [Steven Hartley](https://www.linkedin.com/in/stevenhartley/) to shape this concept both within the specification, but also within implementation.

### Stick through the turbulence

The project you're working a part of may see some ups and downs, may be cancelled outright and those are some very powerful learning moments. There were three major shake-ups that occurred in my time working on this automated driving project which were a test of my commitment and I learned from each.

#### Downselect for planning algorithm team

There were two possible teams which could take up the planning algorithm development. One was based out of the USA, had been part of an R&D effort to build an automated driving system, and had gotten it working in similar driving conditions as our targeted operational design domain (ODD). One was based out of Israel, had been part of a different R&D effort to build an automated driving system, but had thus far only tested their algorithms on a test track, with "perfect" localization using very accurate RTK GNSS and a "near-perfect" map created using said GNSS equipment.

The team based out of the USA had a more traditional view on which planning algorithm to use, whereas the team which was based out of Israel focused on more learning-based approaches, which were a bit more cutting edge. I'm not privvy to the details behind the decision-making process, but both teams make their case and ultimately senior leadership chose to go for the learning-based approach.

The outcome was a number of key middle-level leadership which had worked in automated vehicles shedding from the company, leaving us with less experienced folks that had seen such as system through to production (this is foreshadowing!).

What we found out within the first six months is that a planning system which has undergone in-vehicle testing only on a test track, under ideal conditions, and with using perfect localization and maps is that it's extremely sensitive to noise and errors in maps and localization. While I raised this issue on multiple occasions to the planning team privately, to leadership privately, and also publicly in larger forums, this was not enough to revisit the choice made of which planning algorithm to use. Additionally, much of the more experienced mid-level leadership whose voices might be listened to were no longer with the company.

It's from reflecting back on this experience in 2019 or so, that when joining the effort on Eclipse uProtocol in 2024 I pushed hard to be brought on as a staff-level engineer. Within an organization as large and complex as General Motors, at the end of the day sometimes it truly _is_ your title which will allow you to be in the right meetings and be endowed with enough authority to push for what you believe to be the correct decisions. That designation as a staff engineer helped me to push for including Rust in key places in our infrastructure for Eclipse uProtocol where robustness and reliability were a priority.

#### Workshop

When we began to get production-quality internal deliverables required for the automated driving product before the winter break of 2021, there was a large gap between the expected performance of the planning system and the actual performance. We had been developing for 2.5 years or so using an almost "too accurate" / "too good" mock of what the final production deliverable was that was needed for the planning stack: the map.

Now the problem of the planning stack had senior leadership's attention: we can't ship with the current planning algorithm.

We embarked on a six week series of workshops to find how to best shape the planning algorithm and its inputs to allow for greater flexibility of both. After many conversations and lots of compromises and due in large part to a change in planning team leadership, we were able to come to a clear roadmap of how to change course towards a viable automated driving system.

Hanging in there while I thought the project was not aimed in the right direction due to a single technical team was difficult. I learned that it's not enough for leadership to essentially thumbs-up or thumbs-down proposals without being technically deep enough themselves. I gained a greater sense of patience as I worked through various channels to try to get us back on the right track. And I had built up enough credibility with my manager and other leadership through being a consistent voice on this issue that I was elected as one of the people to join this workshop to reform the project, even though by this point I was "only" an individual contributor.

#### Cancellation

About this time last year in October 2023 senior leadership announced they were ending the automated driving project I had poured six years of my life into. I won't mince words -- this was painful. I had bought my first house while working at General Motors, had started my family and grown it to three wonderful kids.

My manager had found our software a new home, as an improvement to other existing automated driving software currently in production by General Motors. He wanted me to lead one of the efforts to prepare our software to use lower quality and lower resolution inputs. I worked with a small team of engineers in our team and a new upstream team of ours over the next seven months on this project to the point where we had proven out the value of our algorithms on a pre-production vehicle with all of our production inputs.

While this was painful, this was also a learning experience. I got to see my manager at the time quickly pick up and reach out to folks in the business to find another avenue to use the software we had written over those six years. I saw how he tirelessly worked to find where we could bring value and save the positions of all the folks I had worked alongside for years. My appreciation for the broad scope of an engineering manager deepened.

## A next chapter

After riding out the experience for six years on a revolutionary product which became more of an evolutionary addition to an automated driving system, I didn't see much innovation possible within this group I had worked alongside for years.

I had been learning and writing Rust off and on for about six years at this point, so I decided to push for its value proposition within the automated driving domain. While I succeeded in proving to myself that there's a place for Rust in that domain, I didn't convince others due to some very specific point points around Rust's linear algebra library story.

Thanks to the advice by a long-time friend and colleague at General Motors, I found out about the Eclipse uProtocol project where Rust was being actively welcomed as a contribution from the community. I decided to pivot from automated driving to a bit lower in the stack to communications and systems software and have very much enjoyed my time in the world of Eclipse uProtocol. Feel free to check out if Eclipse uProtocol can serve your needs for more reusable software in an open source Software Defined Vehicle stack.

Thanks for taking the time to read! 👋

