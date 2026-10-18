# Scientific Foundations of Data Sonification: Theory, Taxonomy, Psychoacoustics, Mapping Design and Evaluation (for SONARITHM)

> **Method and verification notes (read first).** Every DOI below was checked against the Crossref REST API (api.crossref.org) during this session, and the author, year, venue, volume and pages were cross-checked. Abstracts were pulled from Semantic Scholar where available. I read the full text of these key primary sources: Hermann (2008), Walker & Nees (2011, *Sonification Handbook* ch. 2), Walker (2002), Walker (2007), Walker & Kramer (2005), Smith & Walker (2005), Nees & Walker (2007, 2008), Walker & Mauney (2010, abstract and intro), Dombois (2001), Hermann & Ritter (2002) and Williamson & Murray-Smith (2005). sonification.de (the *Sonification Handbook* site) refused connections throughout the session. Handbook chapters other than ch. 2 are therefore cited from the publisher's verified table of contents and not from their text. Where a numeric value is a standard psychoacoustics figure I could not re-read in full text (the paywalled JASA papers), it is flagged **[std. value, not re-read]**. Items marked **[preprint]** are not peer-reviewed. Inline links use the short form "Author (Year), *Venue*". Full bibliographic records are in the BibTeX section at the end.

---

## 1. Definitions and taxonomy: auditory display, sonification, audification, PMSon, MBS, earcons, auditory icons, spearcons; Hermann's 2008 definition; Kramer; Enge et al.; recent taxonomies

### Takeaway
The field has two definitions that still co-exist. Kramer et al. (1999) define sonification by purpose ("transformation of data relations into perceived relations in an acoustic signal"). Hermann (2008) defines it by method, with four necessary and sufficient conditions: objectivity, systematicity, reproducibility and reusability with different data. Under Hermann's definition audification, earcons, auditory icons, PMSon and MBS all count as sonification *techniques*. Enge et al. (2023) align the vocabulary with visualization theory: time is the *temporal substrate*, sounds are *auditory marks*, and sound attributes are *auditory channels*. That is the most useful vocabulary for a framework, like SONARITHM, that will also emit visuals.

### Cited Findings

**Foundational definitions (Kramer era)**
- The "shortest accepted" definition (Barrass, Kramer et al.) is "Sonification is the use of non-speech audio to convey information." The longer form is "the transformation of data relations into perceived relations in an acoustic signal for the purposes of facilitating communication or interpretation." — [Kramer et al. (1999), *Sonification Report*, ICAD/NSF](https://www.icad.org/websiteV2.0/References/nsf.html), quoted verbatim in [Hermann (2008), *ICAD*](https://www.icad.org/Proceedings/2008/Hermann2008.pdf) and [Walker & Nees (2011), *Sonification Handbook* ch. 2](https://sonification.de/handbook/chapters/chapter2/)
- Hermann traces the field's birth to the first ICAD in 1992, organized by Kramer. The resulting volume *Auditory Display* (Kramer, 1994) is "still one of the most important books in the field." — [Hermann (2008)](https://www.icad.org/Proceedings/2008/Hermann2008.pdf); [Kramer (Ed.) (1994), *Auditory Display*, Addison-Wesley]
- Kramer's *analogic–symbolic continuum* (direct analog representations vs. symbolic codes) is the classic axis for placing techniques. Hermann calls it "secondary" for the definition itself. — [Hermann (2008)](https://www.icad.org/Proceedings/2008/Hermann2008.pdf)
- An *auditory display* is "any display that uses sound to communicate information". Sonification is the subtype that uses non-speech audio. — [Walker & Nees (2011)](https://sonification.de/handbook/chapters/chapter2/)

**Hermann (2008): definition and criteria (verbatim from the paper)**
- "A technique that uses data as input, and generates sound signals (eventually in response to optional additional excitation or triggering) may be called sonification, if and only if
  (C1) The sound reflects objective properties or relations in the input data.
  (C2) The transformation is systematic. This means that there is a precise definition provided of how the data (and optional interactions) cause the sound to change.
  (C3) The sonification is reproducible: given the same data and identical interactions (or triggers) the resulting sound has to be structurally identical.
  (C4) The system can intentionally be used with different data, and also be used in repetition with the same data." — [Hermann (2008), *ICAD 2008*, Paris](https://www.icad.org/Proceedings/2008/Hermann2008.pdf)
- C2 explicitly allows randomness, such as onset jitter that keeps coinciding events perceptible, provided "it is important to declare where and what random elements are used (e.g. by describing the noise distribution)". — [Hermann (2008)](https://www.icad.org/Proceedings/2008/Hermann2008.pdf)
- C3 asks for "structurally identical" output, not sample-identical output: "all possible psychophysical tests should come to identical conclusions." — [Hermann (2008)](https://www.icad.org/Proceedings/2008/Hermann2008.pdf)
- Under this definition, audification, earcons, auditory icons, PMSon and MBS are all sonification techniques. Hermann recommends always using "the most specific term possible" (PMSon, audification, MBS) and using "sonification" with the same generality as "visualization". — [Hermann (2008)](https://www.icad.org/Proceedings/2008/Hermann2008.pdf)
- Hermann's *auditory display* also includes the transmission chain (loudspeakers, headphones, bone conduction) and the user and application context. *Auditory interfaces* additionally include auditory *input* (voice control, query-by-humming). — [Hermann (2008)](https://www.icad.org/Proceedings/2008/Hermann2008.pdf)
- Music versus sonification: data-driven art that violates C2 (implicit rules) or C4 is better called "data-inspired music" or "data-controlled music". — [Hermann (2008)](https://www.icad.org/Proceedings/2008/Hermann2008.pdf)
- Closed-loop taxonomy, in order of increasing interactivity: **Process Monitoring** (least interactive), **Interactive Sonification**, **Navigation**, **Auditory Biofeedback** and **Human Activity**. Interactive Sonification has three sub-types: (a) *triggering*, (b) *parameter adjustment*, (c) *excitatory interaction*. — [Hermann (2008)](https://www.icad.org/Proceedings/2008/Hermann2008.pdf)
- Walker & Nees summarize Hermann's conditions as "data-dependency, objectivity, systematicness, and reproducibility" and note that speech-like sounds such as spearcons challenge the exclusion of speech from sonification taxonomies. — [Walker & Nees (2011)](https://sonification.de/handbook/chapters/chapter2/)

**Techniques**
- **Parameter-Mapping Sonification (PMSon):** "data values are mapped to acoustic attributes of a sound (in other words: the data 'play' an instrument)". — [Hermann (2008)](https://www.icad.org/Proceedings/2008/Hermann2008.pdf). PMSon "tends to result in a lower dimension display than the model-based approaches". — [Walker & Nees (2011)](https://sonification.de/handbook/chapters/chapter2/). Handbook treatment: [Grond & Berger (2011), *Sonification Handbook* ch. 15, pp. 363–397](https://sonification.de/handbook/chapters/chapter15/) (sections on data features, mapping topology, auditory graphs, vowel/formant PMSon and design challenges, per the [publisher TOC](https://logos-verlag.com/tocs/9783832528195toc.pdf)).
- **Model-Based Sonification (MBS):** "the data is used to build an instrument or sound-capable object, while the playing is left to the user". Models "do not make sound at all without external interactions". — [Hermann (2008)](https://www.icad.org/Proceedings/2008/Hermann2008.pdf). Hermann's 2002 thesis formulation: "the sonification is the reaction of the data-driven model to the actions of the user". MBS "tend[s] to involve high data dimensionality and large numbers of data points". — quoted in [Walker & Nees (2011)](https://sonification.de/handbook/chapters/chapter2/); see [Hermann (2002), PhD thesis, Bielefeld](http://sonification.de/publications/Hermann2002-SFE); [Hermann (2011), *Sonification Handbook* ch. 16](https://sonification.de/handbook/chapters/chapter16/).
- **Audification:** "the most prototypical method of direct sonification, whereby waveforms of periodic data are directly translated into sound", which may require time or frequency shifting into the audible range. — [Walker & Nees (2011)](https://sonification.de/handbook/chapters/chapter2/); [Dombois & Eckel (2011), *Sonification Handbook* ch. 12](https://sonification.de/handbook/chapters/chapter12/)
- **Auditory icons:** brief sounds that bear an ecological resemblance to what they represent ("nomic mapping"). — [Walker & Nees (2011)](https://sonification.de/handbook/chapters/chapter2/); [Gaver (1986), *Human–Computer Interaction*](https://doi.org/10.1207/s15327051hci0202_3); ecological "everyday listening" basis in [Gaver (1993), *Ecological Psychology*](https://doi.org/10.1207/s15326969eco0501_1)
- **Earcons:** abstract, symbolic musical motifs. They are more flexible than auditory icons, but "the abstract nature of earcons may require longer learning". — [Walker & Nees (2011)](https://sonification.de/handbook/chapters/chapter2/); [Blattner, Sumikawa & Greenberg (1989), *Human–Computer Interaction*](https://doi.org/10.1207/s15327051hci0401_1); empirical evaluation in [Brewster, Wright & Edwards (1993), *CHI '93*](https://doi.org/10.1145/169059.169179)
- **Spearcons:** produced by time-compressing a spoken phrase. They "can represent anything (like earcons can), but are non-arbitrarily mapped to their concept (like auditory icons)". — [Walker & Nees (2011)](https://sonification.de/handbook/chapters/chapter2/); spearcons improved navigation in advanced auditory menus — [Walker et al. (2013), *Human Factors* 55(1)](https://doi.org/10.1177/0018720812450587)

**Function vs. technique taxonomies**
- Functions: alerts/notifications, status and process monitoring, data exploration, and art/entertainment. Techniques: event-based (PMSon), model-based and continuous (audification). "Event-based approaches are the only ones used for alerts, notifications, alarms, and even status and process monitors". "Data exploration may employ event-based approaches, model-based sonification, or continuous sonification depending upon the specific task". — [Walker & Nees (2011)](https://sonification.de/handbook/chapters/chapter2/)
- Data types should drive the choice of channel. Nominal/categorical data should go to categorically changing acoustic variables such as timbre. Interval data can go to continuous variables such as pitch or loudness (Barrass's recommendation). — [Walker & Nees (2011)](https://sonification.de/handbook/chapters/chapter2/)

**Unified sonification/visualization terminology (Enge et al.)**
- Enge et al. adopt three constructs from visualization theory: the spatial substrate, the visual mark and the visual channel. "We choose time to be the temporal substrate of sonification. Auditory marks are then positioned in time, such as visual marks are positioned in space. Auditory channels are encoded into auditory marks to convey information." — [Enge, Rind, Iber, Höldrich & Aigner (2023), *Personal and Ubiquitous Computing* 27(5):1949–1963](https://doi.org/10.1007/s00779-023-01720-5)
- The journal version extends the Audio Mostly 2021 paper with a discussion of auditory channels, a discussion of *frequency as a potential substrate*, and worked descriptions of existing designs. — [Enge et al. (2023)](https://doi.org/10.1007/s00779-023-01720-5) (via [PMC](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC10589160/)); original: [Enge et al. (2021), *Audio Mostly '21*](https://doi.org/10.1145/3478384.3478415)
- An EuroVis 2024 STAR built on this terminology classifies 57 audiovisual-display papers (2011–2023) by reading level, dataset type, evaluation system and other dimensions. Its co-author network analysis found "individual teams without many interconnections". — [Enge, Elmquist, Caiola, Rönnberg, Rind, Iber et al. (2024), *Computer Graphics Forum* 43(3)](https://doi.org/10.1111/cgf.15114) ([arXiv:2402.16558](https://arxiv.org/abs/2402.16558))

**Other design spaces and taxonomies (2014–2026)**
- *Listening modes × display purposes* as the source of design guidelines for interactive sonification. — [Grond & Hermann (2014), *Organised Sound* 19(1)](https://doi.org/10.1017/S1355771813000393)
- Nees applies Gregor & Jones's eight components of design theory to sonification: purpose and scope, constructs, principles of form and function, artifact mutability, testable propositions, justificatory knowledge, principles of implementation, and expository instantiations. — [Nees (2019), *ICAD 2019*](https://doi.org/10.21785/icad2019.048)
- Erie's grammar defines a design space of tones (periodic wave, sampling, FM/AM synthesizers), encoding channels, auditory legends, and composition operators (sequence, overlay). — [Kim, Kim & Hullman (2024), *CHI '24*](https://doi.org/10.1145/3613904.3642442)
- PMSon mapping functions are characterized "in terms of scaling, transfer function and polarity". Kantan et al. propose the multitrack mixer as a design-platform metaphor (prototype "Mix-N-Map", not yet user-tested). — [Kantan, Dahl & Spaich (2024), *ICAD 2024*](https://doi.org/10.21785/icad2024.034)
- *Signals* represent data structures and *symbols* represent contextual qualities, as a way to structure data-to-display transformations (formative evaluation only, n=2). — [Crozzoli, Muruaga & Magnusson (2025), *ICAD 2025*](https://doi.org/10.21785/icad2025.094)
- A "design model for systemic sonification". — [Seiça, Roque, Martins & Cardoso (2024), *J. Audio Eng. Soc.* 72(9)](https://doi.org/10.17743/jaes.2022.0165) (abstract not retrieved)
- A scale of *intentionality* for communicating social issues through sonification. — [Lenzi & Ciuccarelli (2020), *Big Data & Society*](https://doi.org/10.1177/2053951720944603)
- The Data Sonification Archive corpus (470 projects, 1994–2024) is classified by Goal (art / engagement / research). "The goal of engagement has become more prevalent at the expense of the goal of art, with a large effect size". The authors also track eight published, often-cited definitions of sonification. — [Lindborg, Caiola, Lenzi & Ciuccarelli (2025), *ICAD 2025*](https://doi.org/10.21785/icad2025.017)
- A book-length design framework. — [Worrall (2019), *Sonification Design*, Springer](https://doi.org/10.1007/978-3-030-01497-1)

### Inferences
- SONARITHM's internal model can be organized directly as **substrate (time; optionally frequency) → marks (note events, grains, continuous streams) → channels (pitch, loudness, timbre, pan, duration, rhythm)**, following Enge et al. That gives the Rust core a vocabulary shared with any visual layer and with Erie-style specifications.
- Hermann's C1–C4 translate into engineering requirements: a declarative, serializable mapping specification (C2), deterministic rendering with seeded and declared randomness (C2/C3), and data-agnostic reusable designs (C4).
- SONARITHM should name its modes with Hermann's specific terms ("PMSon", "audification", "MBS") in UI and papers rather than the generic "sonification".

### Gaps
- I could not access the full text of Kramer (1994) or of Handbook chapters 1, 12, 15 and 16 (sonification.de refused connections). Their contents are cited from verified secondary quotations and the publisher's TOC.
- I found no 2020–2026 *consensus* taxonomy that has displaced Hermann (2008). Lindborg et al. (2025) count eight competing published definitions but list them only in the paper body, which I did not retrieve.

---

## 2. Mapping design and psychoacoustics: polarity, scaling, magnitude estimation, why pitch dominates, multivariate mapping, interference, perceptual scales, loudness compensation, JNDs, masking/ASA, temporal limits, context cues, the Nees & Walker model, speech + non-speech

### Takeaway
Pitch dominates because it has fine resolution, a contour that parallels trend, and broad listener agreement. Even so, *polarity and scaling are mapping-specific and population-specific* (Walker 2002, 2007; Walker & Mauney 2010), and designer intuition is a poor predictor of performance (Walker & Kramer 2005). Loudness is a poor primary data channel because it interacts with pitch and is perceived asymmetrically (Neuhoff et al. 2002). Point estimation needs context cues: a dynamic max/min reference tone cut RMS error by about a third, and clicks help timing (Smith & Walker 2005). Brief training (about 20 min) can halve error. Narrow mid-range pitch bands (about 400–1400 Hz) perform as well as wide ones for line charts (Fitzpatrick & Neff 2023).

### Cited Findings

**2a. Why pitch dominates (usage and evidence)**
- Dubus & Bresin reviewed 179 publications from 60 projects and identified 495 mapping occurrences. "Pitch is the most used auditory dimension", used significantly more than all other 29 dimensions. More than half of the 14 most popular mappings involve pitch. Loudness, duration and spatialization are also frequent, and stereo panning accounts for more than half of spatialization uses. — [Dubus & Bresin (2013), *PLoS ONE* 8(12):e82491](https://doi.org/10.1371/journal.pone.0082491)
- Spatial auditory dimensions are used "almost exclusively" for kinematic quantities (location/motion → spatialization). Loudness is used mainly for kinetic quantities. Popular non-pitch mappings follow natural associations, such as distance → loudness (inverse distance law). Density → duration is the one popular mapping that fits neither pattern. — [Dubus & Bresin (2013)](https://doi.org/10.1371/journal.pone.0082491)
- Trend in a sonification "closely parallels the notion of melodic contour in a piece of music". Trend identification is posited as a task the auditory system is particularly suited to. — [Walker & Nees (2011)](https://sonification.de/handbook/chapters/chapter2/)
- Pitch is "generally good for representing temperature, whereas tempo is not as effective". Some dimensions, such as loudness, are "simply not very effective in auditory displays for practical design reasons". — [Walker & Nees (2011)](https://sonification.de/handbook/chapters/chapter2/) citing [Walker (2002)](https://doi.org/10.1037/1076-898X.8.4.211) and [Neuhoff, Kramer & Wayand (2002)](https://doi.org/10.1037/1076-898X.8.1.17)
- In a process-control task, "tempo and onset were generally not as effective as pitch and loudness" for reaction time, because tempo and onset changes "are relatively slow to manifest". — [Walker & Kramer (2005), *ACM TAP* 2(4):407–412](https://doi.org/10.1145/1101530.1101534)
- In an exploratory study with people with visual impairments, participants "rated pitch to be the most intuitive, while the number of tappings and the length of sounds yielded the most accurate perception in decoding data". Data-level perception "might not directly transfer to chart-level perception". — [Wang, Jung & Kim (2022), *Computer Graphics Forum* 41(3):71–83](https://doi.org/10.1111/cgf.14523)

**2b. Polarity, scaling, magnitude estimation (Walker; Walker & Kramer)**
- Method: modulus-free magnitude estimation, where listeners assign numbers to the conceptual magnitude (e.g., "temperature") a sound represents. Polarity is the sign of the regression slope in log–log coordinates and scaling is its magnitude (a Stevens power-law exponent). — [Walker (2002), *J. Exp. Psychol.: Applied* 8(4):211–221](https://doi.org/10.1037/1076-898X.8.4.211); [Walker & Mauney (2010), *ACM TACCESS* 2(3)](https://doi.org/10.1145/1714458.1714459); power-law background in [Stevens (1957), *Psychological Review*](https://doi.org/10.1037/h0046162)
- Walker (2002), Exp. 2 (132 listeners): perceptual pitch-vs-frequency slope m = 0.78; perceived tempo vs tempo m = 0.95; no negative polarities for the perceptual judgments. Conceptual mappings varied. Temperature:frequency had 11/16 positive. Velocity:frequency had 14/16 positive. Dollars:frequency had only 6/16 positive (5 negative, 5 no polarity). All participants judged size:tempo negative (faster = smaller). — [Walker (2002)](https://doi.org/10.1037/1076-898X.8.4.211)
- Walker (2002), Exp. 3 (60 listeners): graphs built with the empirically derived slopes were preferred over shallower or steeper slopes. A "majority-polarity" heuristic (a polarity that gets a majority of responses is "natural") correctly predicted 7 of 10 mappings. "Magnitude estimation is proposed as a design tool … with the level of polarity preference agreement predicting mapping effectiveness." — [Walker (2002)](https://doi.org/10.1037/1076-898X.8.4.211)
- Walker (2007): 209 + 226 undergraduates (a direct replication) and three sound attributes (frequency, tempo, modulation index) × 10 data dimensions. Polarities and slopes were "stable across a direct replication", but "listener expectations depend on both sound and data dimensions in use". Pitch:frequency slope was 0.89 (95% CI 0.71–1.07), tempo 0.85, and brightness:modulation index 0.83. — [Walker (2007), *Applied Cognitive Psychology* 21(5):579–599](https://doi.org/10.1002/acp.1291) (open PDF: [sonify.psych.gatech.edu](http://sonify.psych.gatech.edu/publications/pdfs/2007ACP-Walker.pdf))
- Walker (2007), Exp. 1, frequency (pitch) set, n = 19 per block, counts as positive / none / negative with the majority slope (read from Table 1):

  | Data dimension | +/0/− | Majority slope | Reading |
  |---|---|---|---|
  | Velocity | 18/1/0 | +0.77 | strong positive |
  | Pressure | 13/3/3 | +0.77 | positive |
  | Urgency | 14/3/2 | +0.70 | positive |
  | Temperature | 11/2/6 | +0.65 | positive (majority) |
  | Danger | 10/8/1 | +0.69 | positive |
  | Proximity | 10/6/3 | +0.78 | positive (weak) |
  | Mass | 2/0/17 | −0.91 | strong **negative** |
  | Size | 9/0/10 | −0.87 (neg) / +0.66 (pos) | split, slight negative majority |
  | Number of dollars | 9/2/8 | +1.36 / −1.09 | split, ambiguous |
  | Attractiveness | 3/8/8 | −0.58 | ambiguous, avoid |

  — [Walker (2007)](https://doi.org/10.1002/acp.1291). The 2002 size:tempo result (unanimously negative) was *not* replicated in 2007 (tempo set: 9 positive / 1 none / 9 negative). Conflict noted.
- The "number of dollars" split is explained by mental models. Sighted listeners used an abstract "higher = more" model (positive). Visually impaired listeners used a reality-based model in which a bag of bills makes a "deep thud" (negative). — [Walker (2002)](https://doi.org/10.1037/1076-898X.8.4.211); confirmed by [Walker & Mauney (2010)](https://doi.org/10.1145/1714458.1714459). They found "general agreement about polarities obtained with the two listener populations, with some notable exceptions". Sighted participants were "more likely to intuit split polarities". — [Walker & Nees (2011)](https://sonification.de/handbook/chapters/chapter2/)
- Walker & Kramer (process-control "widget factory", accuracy and RT): "We were surprised to see that the 'Bad' ensemble actually led to the fastest performance." The "Random" ensemble performed best overall and the "Intuitive" ensemble was among the poorest. Flipping a single pitch polarity in the "Bad" ensemble "resulted in a dramatic increase in RT". Conclusion: "data-to-sound mappings that seem intuitive to a sound designer may actually result in less effective performance". Also, "increasing mass is generally best represented by decreasing pitch". — [Walker & Kramer (2005)](https://doi.org/10.1145/1101530.1101534)
- Caution on exponents: "it is important not to over-interpret the exact exponent values reported in any single study … they should be treated as guidance". — [Walker & Nees (2011)](https://sonification.de/handbook/chapters/chapter2/)
- Worked scaling example: Smith & Walker mapped dollars to frequency as $ = c·f^0.9256 (sighted-listener slope from Walker & Lane 2001), with c = 0.0765, so that $50 → 1100 Hz. The resulting range was 200–1900 Hz. — [Smith & Walker (2005), *Applied Cognitive Psychology* 19(8):1065–1087](https://doi.org/10.1002/acp.1146)
- Psychoacoustic channels: a magnitude-estimation study of roughness, noise and pitch for *stress, error and danger* found that "polarity and scaling preference are dependent on the data:sound mapping". It provides polarity and scaling values for direct use. — [Ferguson & Brewster (2018), *CHI '18*](https://doi.org/10.1145/3173574.3174185); see also [Ferguson & Brewster (2017), *ICMI '17*](https://doi.org/10.1145/3136755.3136783)
- Perceived urgency in warnings rises with acoustic parameters such as higher pitch and faster rate (the classic warning-design study). — [Edworthy, Loxley & Dennis (1991), *Human Factors* 33(2)](https://doi.org/10.1177/001872089103300206) **[detail not re-read]**

**2c. Multivariate mapping and cross-dimensional interference**
- Pitch and loudness interact. Three experiments showed that "changes in loudness can influence judgments of pitch change, changes in pitch can influence loudness change, and that increases in loudness are judged to change more than equivalent decreases". In a stock-market sonification, these effects "created perceptual distortions in the data set". — [Neuhoff, Kramer & Wayand (2002), *J. Exp. Psychol.: Applied* 8(1):17–25](https://doi.org/10.1037/1076-898X.8.1.17) (abstract via [Georgia Tech repository listing](https://repository.gatech.edu/entities/publication/b42c37f0-ff76-47b9-833e-24b977198d26))
- Timbre, pitch and loudness behave as interacting (partly integral) dimensions in speeded classification. — [Melara & Marks (1990), *Perception & Psychophysics* 48(2)](https://doi.org/10.3758/BF03207084); [Krumhansl & Iverson (1992), *JEP:HPP* 18(3)](https://doi.org/10.1037/0096-1523.18.3.739) **[detail not re-read]**
- Multidimensional displays transmit more total information than unidimensional ones (classic information-theory result). — [Pollack & Ficks (1954), *JASA* 26(2)](https://doi.org/10.1121/1.1907300) **[detail not re-read]**
- Redundancy can hurt. One study that added panning and spatial elevation on top of frequency "resulted in exceptionally poor performance in the reproduction of simple linear increasing functions". — [Nees & Walker (2007), *ICAD 2007*](https://sonify.psych.gatech.edu/publications/pdfs/2007ICAD-NeesWalker.pdf) (footnote 1)
- Concurrent series rest on auditory streaming theory. "Differences in timbre … and spatial location (or stereo panning) are parameters that sonification designers can often use simply and effectively". Pitch is "a less controllable and less reliable attribute for manipulating segregation" because it is usually busy carrying data. — [Walker & Nees (2011)](https://sonification.de/handbook/chapters/chapter2/), citing [Bregman (1990), *Auditory Scene Analysis*](https://doi.org/10.7551/mitpress/1486.001.0001)
- Identifiability of concurrent earcons falls as their number grows. Timbre differences and onset (staggering) differences partially restore it. — [McGookin & Brewster (2004), *ACM TAP* 1(2):130–155](https://doi.org/10.1145/1024083.1024087) (as summarized in Walker & Nees 2011)
- Musical-instrument timbres (MIDI instruments, often piano) were argued to be "more pleasant and easier to perceive than pure tones". — [Nees & Walker (2007)](https://sonify.psych.gatech.edu/publications/pdfs/2007ICAD-NeesWalker.pdf), citing Brown et al. (2003) and [Brewster et al. (1993)](https://doi.org/10.1145/169059.169179)
- Natural sounds (e.g., birdsong in a forest) were hypothesized to let blind/low-vision listeners hear multiple simultaneous data points. The tool Susurrus was evaluated with 12 BLV participants. — [Hoque, Ehtesham-Ul-Haque, Elmqvist & Billah (2023), *CHI '23*](https://doi.org/10.1145/3544548.3581087)

**2d. Perceptual frequency scales, pitch range and loudness compensation**
- **Mel scale:** pitch magnitude scale from fractionation judgments. — [Stevens, Volkmann & Newman (1937), *JASA* 8(3)](https://doi.org/10.1121/1.1915893). The widely used closed form m = 2595·log10(1 + f/700) is an engineering convention popularized by O'Shaughnessy (1987) and is not given in the 1937 paper. Walker reports magnitude-estimation pitch:frequency exponents of about 0.73–0.89, "slightly higher than the Mel scale". — [Walker (2007)](https://doi.org/10.1002/acp.1291)
- **Bark (critical-band rate):** subdivision of the audible range into critical bands. — [Zwicker (1961), *JASA* 33(2)](https://doi.org/10.1121/1.1908630). Analytical form z = 26.81·f/(1960 + f) − 0.53. — [Traunmüller (1990), *JASA* 88(1)](https://doi.org/10.1121/1.399849)
- **ERB:** ERB(f) = 24.7·(4.37·f/1000 + 1) Hz, and ERB-number = 21.4·log10(4.37·f/1000 + 1). — [Glasberg & Moore (1990), *Hearing Research* 47(1–2)](https://doi.org/10.1016/0378-5955(90)90170-T)
- **Roughness/consonance:** pure-tone pairs sound maximally dissonant at a fraction of a critical bandwidth apart and consonant when separated by more than one critical band. This is the basis for keeping simultaneous data tones more than one critical band apart. — [Plomp & Levelt (1965), *JASA* 38(4)](https://doi.org/10.1121/1.1909741) **[detail not re-read]**. Sensory pleasantness, sharpness, roughness and fluctuation strength models: [Fastl & Zwicker (2007), *Psychoacoustics: Facts and Models*, 3rd ed.](https://doi.org/10.1007/978-3-540-68888-4)
- **Pitch circularity:** Shepard tones produce circular pitch. — [Shepard (1964), *JASA* 36(12)](https://doi.org/10.1121/1.1919362). A Shepard-tone sonification of neonatal SpO2 let participants identify the correct range in 84% of cases (N = 6). — [Schwarz & Ziemer (2019), *ICAD 2019*](https://doi.org/10.21785/icad2019.024)
- **Pitch range guidance:**
  - Scale data to "the range where hearing is most sensitive, say between 1000–5000 Hz" rather than 20 Hz–20 kHz, or use a musical (equal-tempered) model. — [Walker & Nees (2011)](https://sonification.de/handbook/chapters/chapter2/)
  - Brown et al. (2003) recommended limiting to MIDI notes 35–100. Walker & Nees note this leaves "only 65 display points", forcing rounding or pitch-bending, which "tends to take away from the intended musicality". The Sonification Sandbox offered both rounding and exact scaling. — [Walker & Nees (2011)](https://sonification.de/handbook/chapters/chapter2/)
  - In a user study (3 groups × 16 participants), "a shorter range between 400 Hz and 1400 Hz performs just as well as a wider range between 65 Hz and 1480 Hz". — [Fitzpatrick & Neff (2023), *ICAD 2023*](https://doi.org/10.21785/icad2023.1904)
- **Equal loudness:** ISO 226:2023 (Edition 3, 2023-03) gives normal equal-loudness-level contours for pure tones under free-field frontal plane-wave binaural listening. Listeners are otologically normal, aged 18–25. Values differ from the 2003 edition by at most 0.6 dB. — [ISO 226:2023](https://www.iso.org/standard/83117.html) (scope from the [ANSI preview](https://webstore.ansi.org/preview-pages/ISO/preview_ISO+226-2023.pdf)). The underlying dataset: [Suzuki & Takeshima (2004), *JASA* 116(2)](https://doi.org/10.1121/1.1763601). Historical origin: [Fletcher & Munson (1933), *JASA* 5(2)](https://doi.org/10.1121/1.1915637)
- **Does loudness compensation help?** "Using basic amplitude compensation on the wider range has no significant effect on performance or perceived difficulty" (line-chart tasks). — [Fitzpatrick & Neff (2023)](https://doi.org/10.21785/icad2023.1904)

**2e. JNDs and temporal resolution**
- Frequency discrimination as a function of frequency and level. — [Wier, Jesteadt & Green (1977), *JASA* 61(1)](https://doi.org/10.1121/1.381251). The frequency DL is roughly 0.2–0.5% (≈3–9 cents) in the mid range at moderate levels and grows at low levels and above a few kHz **[std. value, not re-read]**.
- Intensity discrimination for pure tones: about 1 dB at low sensation levels, falling toward about 0.5 dB at high levels (the "near miss" to Weber's law). — [Jesteadt, Wier & Green (1977), *JASA* 61(1)](https://doi.org/10.1121/1.381278) **[std. value, not re-read]**
- Duration discrimination of noise and tone bursts. Weber fractions are on the order of 10–20% for durations of hundreds of ms. — [Abel (1972), *JASA* 51(4B)](https://doi.org/10.1121/1.1912963) **[std. value, not re-read]**
- Tempo sensitivity is best (relative JND of a few percent) for inter-onset intervals around 300–800 ms and improves with more intervals ("multiple-look" model). — [Drake & Botte (1993), *Perception & Psychophysics* 54(3)](https://doi.org/10.3758/BF03205262) **[std. value, not re-read]**
- Reporting the temporal *order* of two sounds needs about 20 ms onset separation. — [Hirsh (1959), *JASA* 31(6)](https://doi.org/10.1121/1.1907782) **[std. value, not re-read]**
- Metrical/rhythmic grouping has a lower limit of about 100 ms inter-onset interval (≈10 events/s). — [London (2012), *Hearing in Time*, 2nd ed., OUP](https://doi.org/10.1093/acprof:oso/9780199744374.001.0001). Sensorimotor synchronization breaks down at a similar rate (about 100–120 ms IOI). — [Repp (2005), *Psychonomic Bulletin & Review* 12(6)](https://doi.org/10.3758/BF03206433) **[std. value, not re-read]**
- **Data density in auditory graphs:** Nees & Walker manipulated density at 1, 2, 4 and 8 data points per second together with the number of trend reversals. Density had "a small, but significant, effect" on point-estimation RMS error and listening count. Trend reversals had large effects. The change from one to two points/s "was problematic", particularly with one or three reversals, and zero-reversal graphs stayed at ceiling at all densities. — [Nees & Walker (2008), *ACM TAP* 5(3), Art. 13](https://doi.org/10.1145/1402236.1402237)
- Reference stimulus parameters from a canonical study: 20 tones of 0.5 s each (2 tones/s) over 10 s, 40 ms on/off ramps, about 60 dB SPL over headphones, sine data tones from 200 to 1900 Hz. — [Smith & Walker (2005)](https://doi.org/10.1002/acp.1146)

**2f. Masking, auditory scene analysis and stream segregation**
- Auditory scene analysis (Bregman) explains the pre-attentive formation of auditory objects and streams and "can offer auditory display design heuristics". Designers "should be aware of the psychophysical limitations on judgments of discrimination". — [Walker & Nees (2011)](https://sonification.de/handbook/chapters/chapter2/); [Bregman (1990)](https://doi.org/10.7551/mitpress/1486.001.0001)
- Stream formation depends jointly on frequency separation and presentation rate. A single alternating sequence splits into separate streams when the separation is large and the rate fast (van Noorden's temporal-coherence and fission boundaries). — [Moore & Gockel (2012), *Phil. Trans. R. Soc. B* 367(1591):919–931](https://doi.org/10.1098/rstb.2011.0355) **[std. value, not re-read]**
- Sandor & Lane's "mappable difference" is the absolute response error that must be tolerated to reach a given proportion of correct point estimates. It yields the number of distinct values a mapping can convey at that accuracy. — [Walker & Nees (2011)](https://sonification.de/handbook/chapters/chapter2/)

**2g. Context cues: axes, ticks, reference tones (Smith & Walker)**
- In visual graphs, axes and tick marks enable point estimation. Without context cues, "the accurate extraction of a specific value (i.e., a point estimation task) is impossible", and auditory graphs need equivalent cues. — [Walker & Nees (2011)](https://sonification.de/handbook/chapters/chapter2/)
- Smith & Walker Study 1 (160 undergraduates, estimating dollar values):
  - **x-axis clicks** (10 ms broadband, 1/s, coinciding with every other data tone) reduced error overall (RMS ≈ 14.63 with clicks), "aiding time calibration".
  - A **dynamic y-axis reference tone** played the day's maximum while prices rose and the minimum while they fell, as a triangle wave so it was distinct from the sine data tones. It cut RMS error from 18.09 (no y-context) to **12.14**.
  - A **static reference tone at the opening value** (RMS 17.46) did *not* help significantly.

  — [Smith & Walker (2005)](https://doi.org/10.1002/acp.1146)
- Study 2: clicks did not help with *continuous* stimuli. "Training improved performance overall." Without training, y-axis context brought performance up to the level of trained listeners, so the two effects were "comparable, but not additive". — [Smith & Walker (2005)](https://doi.org/10.1002/acp.1146)
- Clicks at twice the data rate combine "like the major and minor tick marks on the x-axis of a visual graph". — [Walker & Nees (2011)](https://sonification.de/handbook/chapters/chapter2/)
- Point estimation, by task analysis, has five steps: listen; locate the datum in time; estimate the pitch magnitude; compare it to a baseline or reference tone (scaling); report. — [Walker & Nees (2011)](https://sonification.de/handbook/chapters/chapter2/), from Smith & Walker (2005)

**2h. Nees & Walker's conceptual model of auditory graph comprehension**
- The model organizes the variables into three groups: **listener** (perceptual and cognitive abilities, musical experience, visual impairment, training), **task** (point estimation, comparison, trend identification, exploration) and **display** (data properties; mappings, scalings and polarities; timbre; context). It draws on visual graph comprehension models and on hierarchical models of tone-sequence perception (e.g., Deutsch & Feroe; Jones). The authors call it "tentative" and "intentionally" avoid quantifying the relationships. — [Nees & Walker (2007), *ICAD 2007*](https://sonify.psych.gatech.edu/publications/pdfs/2007ICAD-NeesWalker.pdf)
- Musical expertise likely has only a minor effect. Following Watson & Kidd, "the best non-musicians are likely [to] have auditory perceptual abilities on par with the best musicians". Spatial reasoning ability predicted some variance (Walker & Mauney, 2004). — [Walker & Nees (2011)](https://sonification.de/handbook/chapters/chapter2/)
- Training: "a brief training period (around 20 min) can reduce performance error by 50% on a point estimation sonification task" (Walker & Nees 2005b). — [Walker & Nees (2011)](https://sonification.de/handbook/chapters/chapter2/)

**2i. Combining speech with non-speech audio**
- *Audio data narratives* combine textual descriptions with sonification. They were co-designed with screen-reader users and generated automatically by dynamic programming. With narratives, 16 screen-reader users "gain significantly more insights from the data". — [Siu, Kim, O'Modhrain & Follmer (2022), *CHI '22*](https://doi.org/10.1145/3491102.3517678)
- *Infosonics* use spoken introduction and annotation, non-speech audio and sound design. With 18 BLV adults, a COVID-19 infosonic "enabled a clearer mental image than a traditional sonification" and complemented text descriptions. — [Holloway, Goncu, Ilsar, Butler & Marriott (2022), *CHI '22*](https://doi.org/10.1145/3491102.3517465)
- *SpeechTone* is a speech-based mark that encodes data into speech pitch, speed, voice and content, implemented as an extension of Erie. — [Zhao, Lu & Nacenta (2024), *ASSETS '24*](https://doi.org/10.1145/3663548.3688514) ([arXiv:2408.06942](https://arxiv.org/abs/2408.06942))
- Rich screen-reader experiences have three design dimensions: structure, navigation and description, with multiple levels of granularity (13 BVI participants). — [Zong, Lee, Lundgard, Jang, Hajas & Satyanarayan (2022), *CGF* 41(3)](https://doi.org/10.1111/cgf.14519). Related: [Sharif et al. (2022) VoxLens, *CHI '22*](https://doi.org/10.1145/3491102.3517431); [Thompson et al. (2023) Chart Reader, *CHI '23*](https://doi.org/10.1145/3544548.3581186)
- Erie supports auditory legends and uses Web Speech for spoken elements. — [Kim, Kim & Hullman (2024)](https://doi.org/10.1145/3613904.3642442)

### Inferences
- **Scaling law for SONARITHM's "psychophysical" mode:** given a majority exponent m for a data concept, render f = f_ref·(d/d_ref)^(1/m). This is the Smith & Walker (2005) construction inverted. Ship the Walker (2007) table as presets but expose polarity as a one-click toggle, as Walker & Nees (2011) warn against treating exponents as "recipes".
- Nees & Walker (2008) tested at most 8 points/s and London (2012) puts metric grouping at a ~100 ms floor. Together these suggest that above about 8–10 discrete events per second the display should change *mark type* (continuous glide, granular texture or audification) rather than keep emitting discrete notes. This is an inference: no study I found directly measured the discrete-event limit for auditory graphs above 8 points/s.
- Stream segregation (Moore & Gockel 2012) implies that a single series with large, fast pitch jumps can perceptually split into two streams, so its contour is lost. Clamping the range or slowing the rate for volatile data (many reversals per Nees & Walker 2008) protects against this.
- Fitzpatrick & Neff (2023) found no performance gain from amplitude compensation, but compensation still matters for *perceived* equal salience across a range (ISO 226:2023). It is best offered as a default-on perceptual-consistency feature rather than claimed as an accuracy improvement.

### Gaps
- I did not obtain full text for Wier et al. (1977), Jesteadt et al. (1977), Abel (1972), Drake & Botte (1993), Hirsh (1959) or Moore & Gockel (2012), so their numeric JNDs and limits are standard values to verify before publication.
- No study found establishes the *maximum* discriminable data rate for auditory graphs. Nees & Walker (2008) stop at 8 points/s.
- Brown et al. (2003, ICAD) guidelines were cited only via Walker & Nees (2011). The icad.org PDF path I tried returned 404.
- Polarity data come almost entirely from US undergraduates and small BLV samples, with no cross-cultural replication found.

---

## 3. Aesthetics vs. function: musicality, fatigue/annoyance, Neuhoff's "doomed to fail?", Vickers, Supper; pleasant *and* informative

### Takeaway
The field's own critics (Neuhoff 2019; Supper 2014) argue that indiscriminately mixing art and science, and the musical biases of researchers, have held back adoption. Neuhoff recommends designing explicitly for either artistic or scientific goals. Empirical work shows that pleasantness and function can be co-optimized in specific contexts: continuous natural soundscapes for peripheral monitoring (Hildebrandt et al. 2016), discrete square-wave tones preferred by screen-reader users (Sharif et al. 2022), and audiovisual combinations that raise enjoyment (Elmquist et al. 2024). Aesthetics strongly affects long-term acceptance.

### Cited Findings
- **Neuhoff (2019):** "widespread adoption of sonification to display complex data has largely failed to materialize". Impediments: "fundamental perceptual differences between vision and audition, large individual differences in auditory perception, musical biases of sonification researchers, and the interdisciplinary nature of sonification research". "The historical and often indiscriminate mingling of art and science in sonification design may be a root cause". The remedy is designs that "explicitly strive to meet either artistic or scientific goals". — [Neuhoff (2019), *ICAD 2019*, pp. 327–330](https://doi.org/10.21785/icad2019.069)
- A secondary citation of Neuhoff (2019) reports that the word "music" appears in 74% of 1,103 ICAD papers (1994–2018). — via [arXiv:2006.12305](https://arxiv.org/pdf/2006.12305) **[secondhand; not verified in Neuhoff's text]**
- **Supper (2014):** sonification's public appeal rests on the promise of *sublime* experiences, constructed through technological, musical and rhetorical strategies. Public discourse situates sound as immersive and emotional versus "detached" vision, which "reinforc[es] the idea that sound has no place in specialist science". — [Supper (2014), *Social Studies of Science* 44(1)](https://doi.org/10.1177/0306312713496875)
- **Hermann (2008):** music and sonification are both organized sound and sonifications "can be 'heard as' music". Music, however, serves "an underlying artistic level". Sonification-like art without transparent rules is better called "data-inspired music". — [Hermann (2008)](https://www.icad.org/Proceedings/2008/Hermann2008.pdf)
- **Walker & Nees (2011)** list aesthetics, individual differences and training as the major *limiting factors* for sonification deployment (§2.6). — [Walker & Nees (2011)](https://sonification.de/handbook/chapters/chapter2/)
- **Vickers and colleagues:** the chapter "Aesthetics of sonification". — [Vickers, Hogg & Worrall (2017), in *Body, Sound and Space in Music and Beyond*, Routledge, pp. 89–109](https://doi.org/10.4324/9781315569628-6). Handbook chapter covering design, "sensuous perception" and "towards an aesthetic of sonification": [Barrass & Vickers (2011), *Sonification Handbook* ch. 7, pp. 145–171](https://sonification.de/handbook/chapters/chapter7/) (sections from the [publisher TOC](https://logos-verlag.com/tocs/9783832528195toc.pdf)). Middleton et al. cite Vickers, Hogg and Worrall for the challenge of making sonifications "both effective at communicating information and engaging enough to hold sustained attention". — [Middleton et al. (2023)](https://doi.org/10.3389/fdata.2023.1206081)
- **Aesthetic turn:** [Barrass (2012), *AI & Society* 27(2):177–181](https://doi.org/10.1007/s00146-011-0335-5). **Embodied aesthetics:** [Roddy & Furlong (2014), *Organised Sound* 19(1)](https://doi.org/10.1017/S1355771813000423)
- **Monitoring evidence:** with a forest-soundscape continuous sonification, process-monitoring performance was significantly higher than visual-only or visual + auditory alerts. Main-task performance was not significantly affected, and participants found continuous sonification "significantly more helpful and reassuring" (n = 18, within-subjects). — [Hildebrandt, Hermann & Rinderle-Ma (2016), *IJHCS* 95:54–65](https://doi.org/10.1016/j.ijhcs.2016.06.002)
- The same group reports that a first design was effective but "aesthetic qualities were rated poorly". The redesign traded some functional features for long-term compatibility. — [Hildebrandt et al., "Optimizing Aesthetics and Precision in Sonification for Peripheral Process-Monitoring" (Univ. Vienna eprint)](https://eprints.cs.univie.ac.at/4395) (venue details not verified)
- **Screen-reader users' preferences:** 10 participants rated pleasantness, clarity, confidence and overall score. They "preferred distinct non-continuous responses generated using oscillators with square waveforms", and interviews revealed "the need to personalize the sonified responses". — [Sharif, Wang & Muongchan (2022), *ASSETS '22*](https://doi.org/10.1145/3517428.3550360)
- **Musical traits and engagement:** a user-engagement scale was applied to melodic, rhythmic and chordal data-to-music conditions, with reported "benefits and challenges of using musical characteristics in sonifications". — [Middleton et al. (2023), *Frontiers in Big Data* 6:1206081](https://doi.org/10.3389/fdata.2023.1206081)
- **Audiovisual combinations:** in Parallel Chords (n = 35), visualization gave the best sensitivity. Sonification sensitivity was independent of correlation type, and "the combination resulted in increased enjoyability". — [Elmquist et al. (2024), *Personal and Ubiquitous Computing* 28(5)](https://doi.org/10.1007/s00779-024-01795-8). Musical sonification supported visual discrimination of colour intensity. — [Rönnberg (2019), *Behaviour & IT* 38(10)](https://doi.org/10.1080/0144929X.2019.1657952)
- **Natural sounds** were proposed for familiarity and parallel listening. — [Hoque et al. (2023)](https://doi.org/10.1145/3544548.3581087)
- **Annoyance and sensory pleasantness** are modelled through sharpness, roughness, fluctuation strength and loudness. — [Fastl & Zwicker (2007)](https://doi.org/10.1007/978-3-540-68888-4); roughness basis: [Plomp & Levelt (1965)](https://doi.org/10.1121/1.1909741)
- **Public-engagement trend:** "engagement" goals now outweigh "art" in the Data Sonification Archive. — [Lindborg et al. (2025)](https://doi.org/10.21785/icad2025.017)

### Inferences
- Neuhoff's "pick a goal" advice can be implemented as explicit *modes*. An **Analysis mode** uses exact pitch, reference tones, minimal reverb and a declared mapping, prioritizing C1–C4. A **Presentation/Engagement mode** allows scale quantization, harmony, natural-sound timbres and spatial ambience, with the mapping spec still exported for transparency.
- Fatigue and annoyance can be managed with psychoacoustic budgets: cap sharpness and roughness, avoid sustained high-frequency energy, prefer discrete over continuous tones for chart reading (Sharif et al. 2022), and prefer continuous ambient soundscapes for long peripheral monitoring (Hildebrandt et al. 2016). These budgets are a design inference from the cited evidence, not a validated standard.

### Gaps
- I found no controlled study that measures a *trade-off curve* between pleasantness and accuracy for the same data and task.
- No longitudinal (weeks or months) listener-fatigue study for data sonification was found.
- The Vickers & Hogg (2006) "aesthetic perspective space" paper (ICAD 2006) was not retrieved, and Crossref rate-limited the search.

---

## 4. Techniques beyond PMSon: model-based sonification, audification, granular synthesis, spatial audio/HRTF, and monitoring/anomaly detection

### Takeaway
MBS turns data into an excitable virtual object and suits high-dimensional, large-N exploration. Audification suits long, dense, wave-like time series; Dombois used a 2,200× speed-up for seismograms, and seismic categorization accuracies over 90% are reported. Granular synthesis naturally renders probability distributions (Williamson & Murray-Smith 2005). Spatial audio is used mostly for kinematic data and works best as a secondary or segregating channel. For monitoring, continuous soundscapes outperform discrete alerts as a peripheral secondary task.

### Cited Findings

**Model-based sonification**
- The user excites the model and receives acoustic responses determined by its temporal evolution, so "structural information is holistically encoded into the sound signal". Excitatory interaction can exploit fine manipulation (shake, squeeze, tilt). One example is Shoogle, where text messages are overviewed by shaking a phone. — [Hermann (2008)](https://www.icad.org/Proceedings/2008/Hermann2008.pdf)
- **Crystallization sonification:** a crystal grows in high-dimensional data space from a user-selected "condensation nucleus". The sound summarizes the growth process and conveys local and global intrinsic dimensionality and clustering, as an alternative to dendrogram plots. — [Hermann & Ritter (2002), *ICAD 2002*](https://www.icad.org/Proceedings/2002/HermannRitter2002.pdf)
- **Data sonograms and particle trajectory sonification** were introduced in "Listen to your data" (Hermann & Ritter 1999) and are surveyed in Handbook ch. 16. In a data sonogram, a spherical shock wave expands from a user-chosen point and excites each data point's sound as the wavefront passes. In particle trajectory sonification, particles move in a data-derived potential and their motion is rendered as sound. — [Hermann & Ritter (1999), in *Advances in Intelligent Computing and Multimedia Systems*, pp. 189–194]; [Hermann (2011), ch. 16](https://sonification.de/handbook/chapters/chapter16/) **[mechanics described from prior knowledge; not re-read this session]**
- Further MBS models: [Hermann & Ritter (2004), neural gas sonification, *IV 2004*](https://doi.org/10.1109/IV.2004.1320243); overview of sound/meaning in auditory data display: [Hermann & Ritter (2004), *Proc. IEEE* 92(4)](https://doi.org/10.1109/JPROC.2004.825904); closing the interaction loop: [Hunt, Hermann & Pauletto (2004), *IV 2004*](https://doi.org/10.1109/IV.2004.1320244); [Hermann & Hunt (2005), *IEEE MultiMedia* 12(2)](https://doi.org/10.1109/MMUL.2005.26); MBS revisited: [Hermann & Ritter (2005), *ACM TAP* 2(4)](https://doi.org/10.1145/1101530.1101557)
- Recent: *SOMson*, a sonification of multidimensional data in Kohonen maps. — [Linke, Ziemer, Desainte-Catherine & Semal (2024), *ICAD 2024*](https://doi.org/10.21785/icad2024.008). A general-purpose method maps PCA components to harmonic partial amplitudes ("the first components are more relevant and tend to have larger values" in both). — [Dalziel & Lieck (2025), *ICAD 2025*](https://doi.org/10.21785/icad2025.019)
- Interactive sonification of complex data. — [Pauletto & Hunt (2009), *IJHCS* 67(11)](https://doi.org/10.1016/j.ijhcs.2009.05.006)

**Audification**
- Seismic data have been audified to categorize seismic events "with accuracies of over 90%" (Dombois 2002; Speeth 1961). This "may require that the waveforms be frequency- or time-shifted into the range of audible waveforms". — [Walker & Nees (2011)](https://sonification.de/handbook/chapters/chapter2/); origin: [Speeth (1961), *JASA* 33(7)](https://doi.org/10.1121/1.1908843)
- Dombois chose time compression by plain resampling at a factor of **2,200**. The factor balanced (i) overviewing "days and weeks of registration" against (ii) preserving "the acoustic characteristics of a quake". He used no frequency doubling, used the vertical (z) component only, and scaled down the 24/32-bit seismic dynamic range. This compression is about 10× higher than Speeth, Frantti or Hayward used, trading away precise phase (P/S) timing. — [Dombois (2001), *ICAD 2001*](https://www.icad.org/Proceedings/2001/Dombois2001.pdf)
- At 2,200×, near quakes sound like "sharp clicks" and distant ones like "deep rumbles". A distant event stretches from about 0.5 s to a few seconds of audio because of dispersion. Frantti & Leverault (1965) found that trained listeners distinguished quakes from explosions at 67.5% on average. — [Dombois (2001)](https://www.icad.org/Proceedings/2001/Dombois2001.pdf)
- Experts' categorizations and verbal descriptions of audified seismic data can serve as heuristics for geoscience. — [Paté, Boschi, Dubois, Le Carrou & Holtzman (2017), *JASA* 141(3)](https://doi.org/10.1121/1.4978441). Seismology outreach video products: [Kilb et al. (2012), *Seismological Research Letters* 83(2)](https://doi.org/10.1785/gssrl.83.2.281)
- The Handbook audification chapter covers methods, history and "what audification should be used for". — [Dombois & Eckel (2011), ch. 12, pp. 301–324](https://sonification.de/handbook/chapters/chapter12/) (TOC only)

**Granular synthesis for densities and distributions**
- Asynchronous granular synthesis displays time-varying probability distributions. Grains (10–500 ms in general; 80–300 ms in their implementations, with squared-exponential envelopes) are drawn from sources according to a probability distribution, with 100–1,000 grains always active. The result "produces a sound that's incoherent when ambiguity is high, resolving to a clear, distinct sound" as certainty rises. Monte Carlo sampling of future states is sonified the same way. — [Williamson & Murray-Smith (2005), *IEEE MultiMedia* 12(2):45–52](https://doi.org/10.1109/MMUL.2005.37) ([eprint](https://eprints.gla.ac.uk/2924/1/sonification.roderick1.pdf))
- Pulsar catalogue parameters were mapped to grain rate, duration, frequency, panning, amplitude and reverb. — [Helmuth (2024), *JAES* 72(5)](https://doi.org/10.17743/jaes.2022.0147). Synthetic virology data with waveshaping and granular synthesis: [Roddy et al. (2026), *Leonardo* 59(1)](https://doi.org/10.1162/leon.a.2583)

**Spatial audio / HRTF / binaural**
- The minimum audible angle is about 1° for frontal sources. — [Mills (1958), *JASA* 30(4)](https://doi.org/10.1121/1.1909553) **[std. value]**; localization review: [Middlebrooks & Green (1991), *Annu. Rev. Psychol.* 42](https://doi.org/10.1146/annurev.ps.42.020191.001031)
- With non-individualized HRTFs, horizontal localization is broadly preserved but front–back confusions and elevation errors increase. — [Wenzel, Arruda, Kistler & Wightman (1993), *JASA* 94(1)](https://doi.org/10.1121/1.407089) **[detail not re-read]**
- Binaural rendering of Ambisonic PMSon (air-quality data) helped introduce uninitiated listeners. Spatial audio worked best "as an extra factor supplementing the manipulation of other sonic parameters such as frequency", mainly for musically trained listeners, and replicating physical space was difficult. — [Beadling & Vickers (2023), *ICAD 2023*](https://doi.org/10.21785/icad2023.3814)
- Spatialization is used chiefly for kinematic quantities, and stereo panning makes up more than half of uses. — [Dubus & Bresin (2013)](https://doi.org/10.1371/journal.pone.0082491). Spatial location aids stream segregation. — [Walker & Nees (2011)](https://sonification.de/handbook/chapters/chapter2/). Redundant pan + elevation + frequency degraded performance. — [Nees & Walker (2007)](https://sonify.psych.gatech.edu/publications/pdfs/2007ICAD-NeesWalker.pdf)
- Recent spatial sonifications include a CoRoT light-curve ambisonic/binaural design [Riber & Serradilla (2024), *ICAD 2024*](https://doi.org/10.21785/icad2024.010) and spatial climate sonification [Zelada & Çamcı (2023), *ICAD 2023*](https://doi.org/10.21785/icad2023.9190). Three-dimensional psychoacoustic sonification for image-guided surgery: [Ziemer (2023), *ICAD 2023*](https://doi.org/10.21785/icad2023.2324); navigation: [Ziemer & Schultheis (2019), *J. Multimodal User Interfaces* 13(3)](https://doi.org/10.1007/s12193-018-0282-2)

**Monitoring, anomaly detection and peripheral awareness**
- Process monitoring is the least interactive closed loop. Listeners attend passively, and changes in the sound "might attract attention and force the user to act". — [Hermann (2008)](https://www.icad.org/Proceedings/2008/Hermann2008.pdf). Handbook chapter covering types of monitoring, modes of listening, environmental awareness, program execution and pitfalls: [Vickers (2011), ch. 18, pp. 455–491](https://sonification.de/handbook/chapters/chapter18/) (TOC)
- **Network monitoring:** SoNSTAR sonifies all TCP/IP flows by mapping TCP flag combinations to recorded sounds in a soundscape. It "raises situational awareness levels" with "lower workload demands (as measured by the NASA TLX method) than visual techniques". — [Debashi & Vickers (2018), *PLoS ONE* 13(4):e0195948](https://doi.org/10.1371/journal.pone.0195948)
- **Industrial/business processes:** continuous soundscape beats alerts. — [Hildebrandt et al. (2016)](https://doi.org/10.1016/j.ijhcs.2016.06.002)
- **Clinical:** respiratory sonification supports eyes-free monitoring and time-sharing. — [Watson & Sanderson (2004), *Human Factors* 46(3)](https://doi.org/10.1518/hfes.46.3.497.50401); [Sanderson, Watson & Russell (2005), *Anesthesia & Analgesia* 101(1)](https://doi.org/10.1213/01.ANE.0000154080.67496.AE); sonification for complex work domains (dimensions and distractors): [Anderson & Sanderson (2009), *JEP: Applied* 15(3)](https://doi.org/10.1037/a0016329)
- **Classic peripheral awareness:** ARKola bottling-plant simulation with auditory icons. — [Gaver, Smith & O'Shea (1991), *CHI '91*](https://doi.org/10.1145/108844.108857). Audio Aura (background office awareness): [Mynatt, Back, Want, Baer & Ellis (1998), *CHI '98*](https://doi.org/10.1145/274644.274720)
- **Anomaly / signal detection in astronomy:** using astronify's default brightness → pitch mapping, 192 participants counted transit-like signals at SNRs from 3 to 100. At SNR 30–100, both experts and non-experts reached 85–100% correct counting with sonification. At SNR 3–5, performance was at chance with sonification. At SNR 7–10, experts did about 2–3× better with *visuals* but no better than non-experts with sonification. The authors infer that training is needed. — [Tucker Brown, Harrison, Zanella & Trayford (2022), *MNRAS* 516(4):5674–5683](https://doi.org/10.1093/mnras/stac2590)

### Inferences
- **Audification parameterization:** with speed-up factor k, a data band [f1, f2] Hz maps to [k·f1, k·f2] Hz and the playback duration is T/k. For seismic 0.1–10 Hz, k = 2,200 gives 220 Hz–22 kHz, which matches Dombois. SONARITHM should compute k from a target audio band (e.g., 100 Hz–5 kHz) and warn when the data band is too wide to fit. This is derived arithmetic, not a published guideline.
- Granular rendering is a natural SONARITHM mark type for histograms, KDEs, uncertainty bands and point clouds whose density exceeds the discrete-event limit of §2.
- For graphs (nodes and edges), MBS-style excitation is a promising interaction. A data sonogram from a clicked node is one example, with wavefront arrival time proportional to graph distance. This is an untested extrapolation.

### Gaps
- EEG audification and sonification best practices: no peer-reviewed review was retrieved this session (e.g., Väljamäe et al. 2013 ICAD review was not found or verified).
- No empirical guidance was found on recommended audification sample rates or anti-aliasing beyond Dombois (2001).
- Hermann's MBS chapter (ch. 16) and the MBS design guidelines in §16.4 could not be read.

---

## 5. Evaluation methodology: standard tasks, metrics, known problems, rigorous design, shared benchmarks/datasets

### Takeaway
Evaluation remains the field's weakest link. Only 6.1% of mapping occurrences in Dubus & Bresin's corpus were assessed, and a survey of Audio Mostly/ICAD papers found many without any evaluation. The standard empirical toolkit includes point estimation, point comparison, trend identification, structure identification, signal counting/detection and discrimination thresholds. Typical metrics are RMS error, accuracy, RT, times-listened, NASA-TLX, SUS and pleasantness/clarity ratings, with magnitude estimation for mapping design. No widely adopted shared benchmark dataset exists.

### Cited Findings

**Standard tasks**
- **Point estimation** (analytic, single datum); **point comparison** (two or more point estimates plus arithmetic; "empirical investigations to date … have not examined point comparison tasks"); **trend identification** (holistic, global or local); **identification of data structure** (for MBS). — [Walker & Nees (2011)](https://sonification.de/handbook/chapters/chapter2/)
- **Point estimation with RMS error** as the dependent variable (dollar values), including ANCOVA on a pre-test baseline. — [Smith & Walker (2005)](https://doi.org/10.1002/acp.1146); RMS error plus mean number of times listened, and percent-correct trend identification: [Nees & Walker (2008)](https://doi.org/10.1145/1402236.1402237)
- **Signal counting/detection** across SNR levels, comparing sonification, visual and combined modes, with expert vs. non-expert groups and remote testing. — [Tucker Brown et al. (2022)](https://doi.org/10.1093/mnras/stac2590)
- **Discrimination threshold / sensitivity** for correlation strength (choose the strongest of three correlations) across visual, audio and combined displays. — [Elmquist et al. (2024)](https://doi.org/10.1007/s00779-024-01795-8)
- **Speeded control responses** (accuracy and RT) in a process-control simulation. — [Walker & Kramer (2005)](https://doi.org/10.1145/1101530.1101534)
- **Dual-task / peripheral monitoring:** arithmetic main task plus a secondary production-process monitoring task, measuring both. — [Hildebrandt et al. (2016)](https://doi.org/10.1016/j.ijhcs.2016.06.002)
- **Magnitude estimation** for polarity and scaling (the design-phase evaluation). — [Walker (2002)](https://doi.org/10.1037/1076-898X.8.4.211); [Ferguson & Brewster (2018)](https://doi.org/10.1145/3173574.3174185)
- **Insight counts / qualitative insight** with BLV users. — [Siu et al. (2022)](https://doi.org/10.1145/3491102.3517678); mental-image clarity: [Holloway et al. (2022)](https://doi.org/10.1145/3491102.3517465)

**Metrics and instruments**
- NASA-TLX workload. — [Hart & Staveland (1988), *Advances in Psychology* 52](https://doi.org/10.1016/S0166-4115(08)62386-9); used in sonification monitoring by [Debashi & Vickers (2018)](https://doi.org/10.1371/journal.pone.0195948)
- SUS usability. — [Brooke (1996), in *Usability Evaluation in Industry*, Taylor & Francis, pp. 189–194]
- Pleasantness, clarity, confidence, overall score. — [Sharif et al. (2022)](https://doi.org/10.1145/3517428.3550360). User-engagement scale. — [Middleton et al. (2023)](https://doi.org/10.3389/fdata.2023.1206081). Perceived difficulty. — [Fitzpatrick & Neff (2023)](https://doi.org/10.21785/icad2023.1904). Enjoyability. — [Elmquist et al. (2024)](https://doi.org/10.1007/s00779-024-01795-8)
- The Handbook evaluation chapter covers general experimental procedures; data collection for perceptual qualities; analysis of identification, attribute-rating and discrimination data; "distance" data from dissimilarity ratings and sorting (MDS-type analysis); and usability testing / active-use procedures. — [Bonebright & Flowers (2011), *Sonification Handbook* ch. 6, pp. 111–144](https://sonification.de/handbook/chapters/chapter6/) (section list from the [publisher TOC](https://logos-verlag.com/tocs/9783832528195toc.pdf)). Laboratory methods: [Bovermann, Rohrhuber & de Campo (2011), ch. 10](https://sonification.de/handbook/chapters/chapter10/)

**Known methodological problems**
- Rare evaluation: of 495 mapping occurrences, only 30 (6.1%) carried an assessment label (15 "good", 15 "bad"), and only 7 of 60 projects (11.7%) assessed at least one mapping. — [Dubus & Bresin (2013)](https://doi.org/10.1371/journal.pone.0082491)
- A mapping of evaluation methods in Audio Mostly and ICAD classifies them by level of user involvement, the participants' role and author intent. — [Seiça, Roque, Martins & Cardoso (2020), *Audio Mostly '20*](https://doi.org/10.1145/3411109.3411146). A search summary of the PDF reports that 30 of 93 screened papers included no evaluation method. — [author PDF](https://cdv.dei.uc.pt/wp-content/uploads/2020/09/seica2020evaluating.pdf) **[number from search snippet; verify]**
- Samples: research "is most often accomplished with samples of sighted students in academic settings". Most studies use naive listeners, so "performance ceilings for sonifications remain[] speculative". Lab conditions with pure tones may give "misleading design heuristics for less controlled, non-laboratory environments". — [Walker & Nees (2011)](https://sonification.de/handbook/chapters/chapter2/)
- Designer intuition is unreliable, so displays must be tested "with listeners representative of the final users". — [Walker & Kramer (2005)](https://doi.org/10.1145/1101530.1101534). BLV and sighted users differ on some polarities. — [Walker & Mauney (2010)](https://doi.org/10.1145/1714458.1714459)
- Expertise confounds: domain experts' visual training advantage did not transfer to sonification. — [Tucker Brown et al. (2022)](https://doi.org/10.1093/mnras/stac2590)
- Theory deficit: "sonification research and practice continue to be hindered by a lack of theory". — [Nees (2019)](https://doi.org/10.21785/icad2019.048). Individual differences, and art/science mixing as a root cause of failure. — [Neuhoff (2019)](https://doi.org/10.21785/icad2019.069)
- Astronomy: the Audible Universe workshop (AU2, 2022, about 50 experts) began a multidisciplinary discussion of "how to properly design and evaluate sonification tools". — [Misdariis et al. (2023), *ICAD 2023*](https://doi.org/10.21785/icad2023.2124). Of nearly 100 sound-based astronomy projects, "efficacy testing is extremely limited". — [Zanella et al. (2022), *Nature Astronomy* 6](https://doi.org/10.1038/s41550-022-01721-z); [Tucker Brown et al. (2022)](https://doi.org/10.1093/mnras/stac2590)

**Rigor recommendations and remote methods**
- Gamified remote evaluation (CURAT game) "is a reliable means for the remote evaluation of sonification". The game should be kept simple, as many players only use defaults. Results were comparable to lab results, and "interaction tremendously improves the precision". — [Ziemer (2024), *ICAD 2024*](https://doi.org/10.21785/icad2024.007)
- Reproducible experiment platforms for monitoring studies. — [Hildebrandt et al. (2016)](https://doi.org/10.1016/j.ijhcs.2016.06.002); reproducibility as a definitional requirement: [Hermann (2008)](https://www.icad.org/Proceedings/2008/Hermann2008.pdf)
- Validate polarities and slopes with the target listener group, for example through replication or matching tasks. — [Walker (2002)](https://doi.org/10.1037/1076-898X.8.4.211)

**Benchmarks and datasets**
- *SonEX* (Degara, Nagel & Hermann, ICAD 2013) proposed a community evaluation-exchange framework for reproducible sonification. — mentioned in search results; **paper not retrieved this session** (icad.org path 404).
- The *Data Sonification Archive* is a curated corpus of 470 projects (1994–2024), useful as a design-pattern corpus rather than a task benchmark. — [Lindborg et al. (2025)](https://doi.org/10.21785/icad2025.017)
- Synthetic light curves with controlled SNR are a reusable stimulus-generation protocol. — [Tucker Brown et al. (2022)](https://doi.org/10.1093/mnras/stac2590)

### Inferences
- SONARITHM can contribute a missing community asset: an open **benchmark harness**. It would provide parametric stimulus generators (trend reversals × density per Nees & Walker 2008; SNR-controlled transits per Tucker Brown et al. 2022; correlation-strength triplets per Elmquist et al. 2024), standard tasks (point estimation, trend ID, extremum, outlier/signal count) and logged metrics (RMS, accuracy, RT, replays, NASA-TLX, SUS, pleasantness). That would directly answer the evaluation gaps documented by Dubus & Bresin (2013) and Seiça et al. (2020).
- Default study design: within-subjects with counterbalanced mapping order, a pre-test baseline as covariate (Smith & Walker 2005), a brief standardized training block (Walker & Nees 2011), mixed BLV/sighted samples (Walker & Mauney 2010) and shared stimuli. This is assembled from the cited practices, not a published standard.

### Gaps
- I found no 2023–2026 systematic review dedicated to sonification *evaluation methodology*, and no shared, widely adopted benchmark dataset with tasks and ground truth.
- No sonification-specific standardized questionnaire (comparable to SUS for usability) was identified.

---

## 6. Recent advances 2022–2026 and open problems: declarative grammars (Erie), Umwelt, ICAD 2023–2026, ML/LLMs, scientific discovery

### Takeaway
2022–2026 brought (i) declarative, multimodal specification (Erie, CHI 2024; Umwelt, CHI 2024; SpeechTone, ASSETS 2024), (ii) a wave of open toolkits (STRAUSS, sonecules, Siren, Ossia Score, Pya AGen, Pixasonics, Highcharts Sonification Studio) and (iii) heavy astronomy uptake. AI/ML for sonification is an explicitly named but nascent agenda (Hommel & Hermann 2025; Budziszewski & Groß-Vogt 2025). I found no peer-reviewed controlled evaluation of LLM-generated sonification mappings.

### Cited Findings

**Grammars and multimodal authoring**
- **Erie** is a declarative grammar for data sonification. It covers extensible tone designs (periodic wave, sampling, FM/AM synthesizers), encoding channels, auditory legends, and sequencing and overlaying. It compiles to Web Audio and Web Speech, replicates prior research prototypes, and comes with a design gallery. The authors list interactive sonification and other audio environments as future work. — [Kim, Kim & Hullman (2024), *CHI '24*](https://doi.org/10.1145/3613904.3642442) ([arXiv:2402.00156](https://arxiv.org/abs/2402.00156))
- **Umwelt** is an authoring environment in which visualization, sonification and textual description are co-equal representations derived from a shared data and field specification, so no modality is privileged. It was evaluated with expert screen-reader users. — [Zong, Pedraza Pineros, Chen, Hajas & Satyanarayan (2024), *CHI '24*](https://doi.org/10.1145/3613904.3641996) **[summary from prior knowledge of the abstract; full text not retrieved]**
- **SpeechTone** extends Erie with speech marks. — [Zhao, Lu & Nacenta (2024)](https://doi.org/10.1145/3663548.3688514)

**Toolkits (ICAD 2021–2025 and JOSS)**
- STRAUSS (Python): [Trayford & Harrison (2023), *ICAD 2023*](https://doi.org/10.21785/icad2023.1978); [Trayford et al. (2025), *JOSS* 10(109):7875](https://doi.org/10.21105/joss.07875)
- sonecules ("design-once-use-many" reusable designs on mesonic, rendering-platform agnostic): [Reinsch & Hermann (2023), *ICAD 2023*](https://doi.org/10.21785/icad2023.5580). Pya AGen audio generators: [Born & Hermann (2025), *ICAD 2025*](https://doi.org/10.21785/icad2025.021)
- Siren (web): [Peng, Choi & Berger (2023), *ICAD 2023*](https://doi.org/10.21785/icad2023.6167). Ossia Score sonification cell: [Poret et al. (2023), *ICAD 2023*](https://doi.org/10.21785/icad2023.4039). Pixasonics image sonification: [Laczko & Jensenius (2025), *ICAD 2025*](https://doi.org/10.21785/icad2025.074). Highcharts Sonification Studio: [Cantrell, Walker & Moseng (2021), *ICAD 2021*](https://doi.org/10.21785/icad2021.005)
- A history of active and legacy sonification tools from 1990 to the present, organized into five user scenarios, identifies sustainability challenges and "outlines a research agenda that considers the application of machine learning for generating intuitive and adaptive sonifications". — [Budziszewski & Groß-Vogt (2025), *ICAD 2025*](https://doi.org/10.21785/icad2025.071)

**AI / ML / LLMs**
- AI's "application in sonification remains limited". The authors present a prototype for AI-supported PMSon on the sc3nb / mesonic / sonecules stack, released as open source, and discuss prerequisites for AI in "incremental design/interaction loops". — [Hommel & Hermann (2025), *ICAD 2025*](https://doi.org/10.21785/icad2025.037)
- The Georgia Tech Sonification Lab's publication list includes an ICAD 2026 paper on AI-driven sonification design (Chang, Snook, Ertzinger & Walker), seen in search results. — [Sonification Lab publications](https://sonify.psych.gatech.edu/publications/) **[title and content not verified]**

**Scientific discovery (astronomy and physics)**
- Nearly 100 sound-based astronomy projects were identified, motivated by discovery in multidimensional data, education and engagement, and BLV access. The authors discuss "current limitations and challenges". — [Zanella et al. (2022), *Nature Astronomy* 6:1241–1248](https://doi.org/10.1038/s41550-022-01721-z)
- Recent ICAD examples: the Sonified Hertzsprung–Russell diagram [Huppenkothen et al. (2023)](https://doi.org/10.21785/icad2023.6263); TESS validation time series [Riber & Garcia (2022)](https://doi.org/10.21785/icad2022.024); CoRoT ambisonics [Riber & Serradilla (2024)](https://doi.org/10.21785/icad2024.010); nanoscale spin waves [Pile et al. (2024)](https://doi.org/10.21785/icad2024.024); 2D quantum systems [Müller, Freye & Hermann (2025)](https://doi.org/10.21785/icad2025.030); biology visualization (Birdsongification) [Elmquist et al. (2024)](https://doi.org/10.21785/icad2024.006)
- Efficacy: high-SNR transits are detectable by ear, but moderate-SNR detection needs training. — [Tucker Brown et al. (2022)](https://doi.org/10.1093/mnras/stac2590)

**Conferences**
- ICAD 2023 (28th) proceedings carry DOIs 10.21785/icad2023.*. ICAD 2024 (29th) is listed in the Georgia Tech repository. — [ICAD 2024 proceedings](https://repository.gatech.edu/entities/publication/8ac0082f-eb1e-4e09-9468-8b5e02a9f451)
- ICAD 2025 (30th) was held in Coimbra, Portugal, co-located with Audio Mostly as "AM.ICAD" (June 30 – July 4, 2025). Editors: Cardoso, Vickers, Martins and Roddy. Licence: CC BY-NC 4.0. — [ICAD 2025 proceedings](https://repository.gatech.edu/entities/publication/5a7af9ce-acb5-4d6f-bebd-cf5daf692609); [AM.ICAD call](https://amicad2025.dei.uc.pt/call-icad.html)
- ICAD 2026 (31st) was announced in a distributed format across three hubs, July 28–31, 2026. No proceedings were found yet. — [icad.org](https://icad.org)
- A notable ICAD 2023 perception result for default pitch range. — [Fitzpatrick & Neff (2023)](https://doi.org/10.21785/icad2023.1904)

**Open problems the field names**
- Lack of adoption; vision/audition differences; individual differences; musical bias; art/science conflation. — [Neuhoff (2019)](https://doi.org/10.21785/icad2019.069)
- Lack of theory, with eight design-theory components as requirements. — [Nees (2019)](https://doi.org/10.21785/icad2019.048)
- Context-cue configurations underexplored ("Researchers have only scratched the surface"); training methods; individual differences; data properties (density, volatility) vs. best mapping. — [Walker & Nees (2011)](https://sonification.de/handbook/chapters/chapter2/)
- Evaluation scarcity. — [Dubus & Bresin (2013)](https://doi.org/10.1371/journal.pone.0082491); [Zanella et al. (2022)](https://doi.org/10.1038/s41550-022-01721-z)
- Fragmented community (few cross-team links). — [Enge et al. (2024)](https://doi.org/10.1111/cgf.15114)
- Tool sustainability and machine-learning integration. — [Budziszewski & Groß-Vogt (2025)](https://doi.org/10.21785/icad2025.071); [Hommel & Hermann (2025)](https://doi.org/10.21785/icad2025.037)
- A shift toward engagement goals. — [Lindborg et al. (2025)](https://doi.org/10.21785/icad2025.017). Public discourse sidelines sound from specialist science. — [Supper (2014)](https://doi.org/10.1177/0306312713496875)
- Software support "not commensurate" with research on effective design. — [Kim, Kim & Hullman (2024)](https://doi.org/10.1145/3613904.3642442)

### Inferences
- A Rust-core engine that consumes an Erie-compatible (or Erie-inspired) JSON spec would align SONARITHM with the most-cited recent grammar while adding native, offline, real-time and graph-aware rendering. Graphs and networks are absent from Erie's published gallery scope as summarized in its abstract.
- A clear publishable niche: **controlled evaluation of LLM-suggested mappings against Walker-style empirical polarity priors.** I found no peer-reviewed study of this.

### Gaps
- No peer-reviewed LLM-for-sonification evaluation was found for 2023–2026. LLM-adjacent hits were non-sonification or 2026 abstracts and preprints that I did not verify.
- ICAD 2026 papers were not yet indexed in Crossref (only DOIs up to icad2025.* appeared).
- Umwelt's full text was not read this session.

---

## 7. Design rules for SONARITHM

### Takeaway
The rules below turn the evidence into defaults. Each is traced to sources. "(Inference)" marks rules that extrapolate beyond direct evidence and should be validated with SONARITHM's own benchmark harness.

### Cited Findings (numbered rules)

**A. Definitions, specification and reproducibility**
1. **Every render must be fully specified by a serializable mapping spec plus data.** Export it alongside the audio to satisfy systematicity and reproducibility (C2/C3). — [Hermann (2008)](https://www.icad.org/Proceedings/2008/Hermann2008.pdf)
2. **All randomness (onset jitter, grain scheduling, noise) is seeded and declared in the spec**, including its distribution and purpose. — [Hermann (2008)](https://www.icad.org/Proceedings/2008/Hermann2008.pdf)
3. **Designs must be data-agnostic and reusable** (C4). Ship templates in the style of "sonecules" that can be re-applied to any dataset with a compatible schema. — [Hermann (2008)](https://www.icad.org/Proceedings/2008/Hermann2008.pdf); [Reinsch & Hermann (2023)](https://doi.org/10.21785/icad2023.5580)
4. **Use the Enge et al. vocabulary in the API:** `substrate` (time; optional frequency), `mark` (note, glide, grain, sample, speech), `channel` (pitch, loudness, timbre, pan, duration, rate). — [Enge et al. (2023)](https://doi.org/10.1007/s00779-023-01720-5)
5. **Interoperate with Erie-like declarative specs**, including tone definitions, encodings, auditory legends and sequence/overlay composition. — [Kim, Kim & Hullman (2024)](https://doi.org/10.1145/3613904.3642442)
6. **Label modes with specific technique names** (PMSon, Audification, MBS, Earcon/Auditory-icon events). — [Hermann (2008)](https://www.icad.org/Proceedings/2008/Hermann2008.pdf)

**B. Pitch mapping defaults**
7. **The primary quantitative channel is pitch** for single-series values and trends. — [Dubus & Bresin (2013)](https://doi.org/10.1371/journal.pone.0082491); [Walker & Kramer (2005)](https://doi.org/10.1145/1101530.1101534); [Wang et al. (2022)](https://doi.org/10.1111/cgf.14523)
8. **Default pitch range is about 400–1400 Hz (≈MIDI 67–89, under two octaves)** for line-chart PMSon. Allow expansion to about 65–1480 Hz (MIDI 36–90) or MIDI 35–100 when more resolution is needed. — [Fitzpatrick & Neff (2023)](https://doi.org/10.21785/icad2023.1904); [Walker & Nees (2011) citing Brown et al. (2003)](https://sonification.de/handbook/chapters/chapter2/)
9. **Map in log-frequency (semitones/cents) by default.** Offer mel, ERB-rate and Bark options for perceptually uniform spacing. The formulas are in §2d. — [Glasberg & Moore (1990)](https://doi.org/10.1016/0378-5955(90)90170-T); [Traunmüller (1990)](https://doi.org/10.1121/1.399849); [Stevens et al. (1937)](https://doi.org/10.1121/1.1915893)
10. **Analysis mode uses continuous (unquantized) pitch.** Presentation mode may snap to a scale. Expose the trade-off: resolution vs. musicality. — [Walker & Nees (2011)](https://sonification.de/handbook/chapters/chapter2/)
11. **Psychophysical scaling option:** f = f_ref·(d/d_ref)^(1/m), using a concept-specific exponent m (e.g., m ≈ 0.93 for dollars in Smith & Walker; Walker 2007 table). Label exponents as guidance, not truth. — [Smith & Walker (2005)](https://doi.org/10.1002/acp.1146); [Walker (2007)](https://doi.org/10.1002/acp.1291); [Walker & Nees (2011)](https://sonification.de/handbook/chapters/chapter2/)

**C. Polarity**
12. **Positive polarity (higher value → higher pitch) by default** for velocity, pressure, temperature, urgency, danger, proximity and generic abstract quantities. — [Walker (2002)](https://doi.org/10.1037/1076-898X.8.4.211); [Walker (2007)](https://doi.org/10.1002/acp.1291)
13. **Negative polarity by default for mass** (strong) **and size** (slight majority; split). — [Walker (2007)](https://doi.org/10.1002/acp.1291); [Walker & Kramer (2005)](https://doi.org/10.1145/1101530.1101534)
14. **Flag ambiguous concepts (money/dollars, attractiveness)** in the UI. Recommend user testing, and note that BLV listeners favoured *negative* polarity for dollars. — [Walker (2002)](https://doi.org/10.1037/1076-898X.8.4.211); [Walker & Mauney (2010)](https://doi.org/10.1145/1714458.1714459)
15. **Always provide a one-click polarity flip and persist it per user profile.** Designer intuition is an unreliable guide. — [Walker & Kramer (2005)](https://doi.org/10.1145/1101530.1101534)

**D. Context cues (auditory axes)**
16. **Add an x-axis tick track:** short (≈10 ms) broadband clicks at major ticks, with optional minor ticks at 2× rate, for discrete-note graphs. Disable by default for continuous glides, where clicks did not help. — [Smith & Walker (2005)](https://doi.org/10.1002/acp.1146); [Walker & Nees (2011)](https://sonification.de/handbook/chapters/chapter2/)
17. **Add a dynamic y-axis reference tone** in a timbre distinct from the data tones (e.g., triangle vs. sine). It sounds the series maximum while values rise and the minimum while they fall. Do not rely on a static "first value" reference tone. — [Smith & Walker (2005)](https://doi.org/10.1002/acp.1146)
18. **Provide spoken auditory legends and axis labels** (range, units, polarity) before playback, and support hybrid speech + sonification narratives. — [Kim, Kim & Hullman (2024)](https://doi.org/10.1145/3613904.3642442); [Siu et al. (2022)](https://doi.org/10.1145/3491102.3517678); [Holloway et al. (2022)](https://doi.org/10.1145/3491102.3517465)

**E. Timing and density**
19. **Default overview rate is about 2–4 data points/s; never exceed ~8 discrete points/s without a warning.** Studied densities span 1–8 points/s. — [Nees & Walker (2008)](https://doi.org/10.1145/1402236.1402237); [Smith & Walker (2005)](https://doi.org/10.1002/acp.1146)
20. **Above about 10 events/s (IOI < ~100 ms), switch mark type** to a continuous glide, granular texture or audification instead of discrete notes. — (Inference from) [London (2012)](https://doi.org/10.1093/acprof:oso/9780199744374.001.0001); [Repp (2005)](https://doi.org/10.3758/BF03206433); [Nees & Walker (2008)](https://doi.org/10.1145/1402236.1402237)
21. **Slow down or smooth volatile series automatically.** Many trend reversals degrade performance more than density does. — [Nees & Walker (2008)](https://doi.org/10.1145/1402236.1402237); [Walker & Nees (2011)](https://sonification.de/handbook/chapters/chapter2/) (pre-processing and volatility reduction)
22. **Use onset/offset ramps** (≈40 ms in the reference study; never zero) to avoid clicks. — [Smith & Walker (2005)](https://doi.org/10.1002/acp.1146)

**F. Loudness, timbre, multivariate and streams**
23. **Do not use loudness as the primary channel for precise values.** Keep loudness constant while pitch carries data, because pitch and loudness interact and rising loudness is overestimated. — [Neuhoff, Kramer & Wayand (2002)](https://doi.org/10.1037/1076-898X.8.1.17); [Walker & Nees (2011)](https://sonification.de/handbook/chapters/chapter2/)
24. **Apply equal-loudness compensation (ISO 226:2023 contours) by default for perceptual consistency** across wide pitch ranges. Do not claim it improves trend accuracy. — [ISO 226:2023](https://www.iso.org/standard/83117.html); [Fitzpatrick & Neff (2023)](https://doi.org/10.21785/icad2023.1904)
25. **Map categorical variables (series identity, class labels) to timbre.** Map interval/ratio variables to pitch, duration or rate. — [Walker & Nees (2011)](https://sonification.de/handbook/chapters/chapter2/)
26. **Separate concurrent series with timbre + stereo position + staggered onsets.** Limit to about 3–4 simultaneous streams by default. — [Walker & Nees (2011)](https://sonification.de/handbook/chapters/chapter2/); [McGookin & Brewster (2004)](https://doi.org/10.1145/1024083.1024087) (the 3–4 cap is an inference)
27. **Avoid redundant spatial encodings** (pan + elevation + pitch on the same variable) in analytic mode. — [Nees & Walker (2007)](https://sonify.psych.gatech.edu/publications/pdfs/2007ICAD-NeesWalker.pdf)
28. **Keep simultaneous tones more than one critical band (≈1 ERB) apart** to limit roughness and masking. Use the ERB/Bark utilities from Rule 9. — (Inference from) [Plomp & Levelt (1965)](https://doi.org/10.1121/1.1909741); [Glasberg & Moore (1990)](https://doi.org/10.1016/0378-5955(90)90170-T)
29. **Default timbre is a harmonic instrument-like tone** (e.g., piano/mallet-like), with pure sine available as an option. For BLV chart reading, offer discrete square-wave tones as a preset. — [Nees & Walker (2007)](https://sonify.psych.gatech.edu/publications/pdfs/2007ICAD-NeesWalker.pdf); [Sharif et al. (2022)](https://doi.org/10.1145/3517428.3550360)
30. **Use spatial position (stereo; optional binaural HRTF) primarily for kinematic/positional variables or for stream segregation.** Warn that generic HRTFs increase front–back confusion. — [Dubus & Bresin (2013)](https://doi.org/10.1371/journal.pone.0082491); [Wenzel et al. (1993)](https://doi.org/10.1121/1.407089); [Beadling & Vickers (2023)](https://doi.org/10.21785/icad2023.3814)

**G. Technique modes**
31. **Audification mode:** compute the speed-up k from the data band to a target audio band (show k, duration T/k and the audible band). Default to resampling (no pitch-shifting) and include dynamic-range scaling. For seismic presets, k ≈ 2,200. — [Dombois (2001)](https://www.icad.org/Proceedings/2001/Dombois2001.pdf); [Walker & Nees (2011)](https://sonification.de/handbook/chapters/chapter2/)
32. **Granular mode for distributions, uncertainty and dense point clouds:** grain probability follows data density, grains last 80–300 ms with smooth envelopes, and 100–1,000 grains are active at once. — [Williamson & Murray-Smith (2005)](https://doi.org/10.1109/MMUL.2005.37)
33. **MBS mode for high-dimensional and graph data:** user-excited models (data-sonogram style wavefront from a selected node; crystallization growth from a seed). Sound should only occur in response to interaction. — [Hermann (2008)](https://www.icad.org/Proceedings/2008/Hermann2008.pdf); [Hermann & Ritter (2002)](https://www.icad.org/Proceedings/2002/HermannRitter2002.pdf)
34. **High-D quick-look preset:** map PCA components to harmonic partial amplitudes. — [Dalziel & Lieck (2025)](https://doi.org/10.21785/icad2025.019)
35. **Monitoring mode uses a continuous ambient soundscape** (natural sounds are a good default) with event-driven salience changes, rather than discrete alarms alone. Reserve urgency cues (higher pitch, faster rate) for true alarms. — [Hildebrandt et al. (2016)](https://doi.org/10.1016/j.ijhcs.2016.06.002); [Debashi & Vickers (2018)](https://doi.org/10.1371/journal.pone.0195948); [Hoque et al. (2023)](https://doi.org/10.1145/3544548.3581087); [Edworthy et al. (1991)](https://doi.org/10.1177/001872089103300206)

**H. Aesthetics and users**
36. **Separate "Analysis" and "Presentation" modes explicitly** and record the mode in the spec. — [Neuhoff (2019)](https://doi.org/10.21785/icad2019.069); [Hermann (2008)](https://www.icad.org/Proceedings/2008/Hermann2008.pdf)
37. **Ship a built-in 10–20 minute interactive training/tutorial.** About 20 minutes of training halved point-estimation error, and training ≈ y-axis context. — [Walker & Nees (2011)](https://sonification.de/handbook/chapters/chapter2/); [Smith & Walker (2005)](https://doi.org/10.1002/acp.1146); [Tucker Brown et al. (2022)](https://doi.org/10.1093/mnras/stac2590)
38. **Support per-user personalization** (timbre, discrete/continuous, polarity, range, rate) and treat BLV users as a primary design population. — [Sharif et al. (2022)](https://doi.org/10.1145/3517428.3550360); [Walker & Mauney (2010)](https://doi.org/10.1145/1714458.1714459)
39. **Psychoacoustic comfort budget:** limit sustained sharpness and roughness and avoid high-frequency-heavy timbres in long sessions. — (Inference from) [Fastl & Zwicker (2007)](https://doi.org/10.1007/978-3-540-68888-4); [Hildebrandt et al. (2016)](https://doi.org/10.1016/j.ijhcs.2016.06.002)

**I. Evaluation (for SONARITHM's papers)**
40. **Evaluate with standard tasks and metrics:** point estimation (RMS), trend ID (accuracy), extremum/outlier/signal detection (hit rate by SNR), discrimination thresholds, RT, replays, NASA-TLX, SUS and pleasantness/clarity. Include BLV participants and a training arm. — [Walker & Nees (2011)](https://sonification.de/handbook/chapters/chapter2/); [Nees & Walker (2008)](https://doi.org/10.1145/1402236.1402237); [Tucker Brown et al. (2022)](https://doi.org/10.1093/mnras/stac2590); [Elmquist et al. (2024)](https://doi.org/10.1007/s00779-024-01795-8); [Hart & Staveland (1988)](https://doi.org/10.1016/S0166-4115(08)62386-9); [Sharif et al. (2022)](https://doi.org/10.1145/3517428.3550360)
41. **Use magnitude estimation to set polarity and scaling for any new data concept** SONARITHM targets (e.g., graph centrality, edge weight). Adopt a mapping only when one polarity has a clear majority. — [Walker (2002)](https://doi.org/10.1037/1076-898X.8.4.211); [Ferguson & Brewster (2018)](https://doi.org/10.1145/3173574.3174185)
42. **Publish stimuli, specs and analysis code**, and consider gamified remote data collection for scale. — [Hermann (2008)](https://www.icad.org/Proceedings/2008/Hermann2008.pdf); [Ziemer (2024)](https://doi.org/10.21785/icad2024.007)

### Inferences
- Rules 20, 26 (the stream cap), 28, 31 (the generic k-from-band computation), 33 (applying MBS to graphs) and 39 are extrapolations. They are good candidates for SONARITHM's first empirical paper: a factorial study of rate × reversals × pitch range × context cues, with BLV and sighted participants.
- Graph-specific concepts such as node degree, centrality, edge weight and path length have *no* published polarity data I could find. Rule 41 is therefore not optional for SONARITHM's graph features.

### Gaps
- No empirical source was found for an upper bound on concurrent streams in data sonification (as opposed to concurrent earcons).
- No validated psychoacoustic "comfort budget" thresholds for long sonification sessions were found.

---

## 8. BibTeX reference list

> Entries with `doi` were verified against Crossref in this session. Entries without a DOI either have none (books, standards, older ICAD papers) or I could not verify one. Those carry a stable `url`. `note` fields mark preprints, unverified details or items I did not retrieve.

```bibtex
@techreport{kramer1999sonification,
  author      = {Kramer, Gregory and Walker, Bruce N. and Bonebright, Terri and Cook, Perry and Flowers, John H. and Miner, Nadine and Neuhoff, John G.},
  title       = {Sonification Report: Status of the Field and Research Agenda},
  institution = {International Community for Auditory Display (ICAD), prepared for the National Science Foundation},
  address     = {Santa Fe, NM},
  year        = {1999},
  url         = {https://www.icad.org/websiteV2.0/References/nsf.html},
  note        = {Author list abbreviated as in Nees \& Walker (2007) reference [1]}
}

@book{kramer1994auditory,
  editor    = {Kramer, Gregory},
  title     = {Auditory Display: Sonification, Audification, and Auditory Interfaces},
  series    = {Santa Fe Institute Studies in the Sciences of Complexity, Proceedings Vol. XVIII},
  publisher = {Addison-Wesley},
  address   = {Reading, MA},
  year      = {1994}
}

@inproceedings{hermann2008taxonomy,
  author    = {Hermann, Thomas},
  title     = {Taxonomy and Definitions for Sonification and Auditory Display},
  booktitle = {Proceedings of the 14th International Conference on Auditory Display (ICAD 2008)},
  editor    = {Susini, Patrick and Warusfel, Olivier},
  publisher = {IRCAM},
  address   = {Paris, France},
  year      = {2008},
  url       = {https://www.icad.org/Proceedings/2008/Hermann2008.pdf}
}

@book{hermann2011handbook,
  editor    = {Hermann, Thomas and Hunt, Andy and Neuhoff, John G.},
  title     = {The Sonification Handbook},
  publisher = {Logos Verlag},
  address   = {Berlin},
  year      = {2011},
  isbn      = {978-3-8325-2819-5},
  url       = {https://sonification.de/handbook/}
}

@incollection{walker2011theory,
  author    = {Walker, Bruce N. and Nees, Michael A.},
  title     = {Theory of Sonification},
  booktitle = {The Sonification Handbook},
  editor    = {Hermann, Thomas and Hunt, Andy and Neuhoff, John G.},
  publisher = {Logos Verlag},
  address   = {Berlin},
  year      = {2011},
  pages     = {9--39},
  url       = {https://sonification.de/handbook/chapters/chapter2/}
}

@incollection{carlile2011psychoacoustics,
  author    = {Carlile, Simon},
  title     = {Psychoacoustics},
  booktitle = {The Sonification Handbook},
  editor    = {Hermann, Thomas and Hunt, Andy and Neuhoff, John G.},
  publisher = {Logos Verlag},
  address   = {Berlin},
  year      = {2011},
  pages     = {41--61},
  url       = {https://sonification.de/handbook/chapters/chapter3/}
}

@incollection{neuhoff2011perception,
  author    = {Neuhoff, John G.},
  title     = {Perception, Cognition and Action in Auditory Displays},
  booktitle = {The Sonification Handbook},
  editor    = {Hermann, Thomas and Hunt, Andy and Neuhoff, John G.},
  publisher = {Logos Verlag},
  address   = {Berlin},
  year      = {2011},
  pages     = {63--85},
  url       = {https://sonification.de/handbook/chapters/chapter4/}
}

@incollection{bonebright2011evaluation,
  author    = {Bonebright, Terri L. and Flowers, John H.},
  title     = {Evaluation of Auditory Display},
  booktitle = {The Sonification Handbook},
  editor    = {Hermann, Thomas and Hunt, Andy and Neuhoff, John G.},
  publisher = {Logos Verlag},
  address   = {Berlin},
  year      = {2011},
  pages     = {111--144},
  url       = {https://sonification.de/handbook/chapters/chapter6/}
}

@incollection{barrass2011design,
  author    = {Barrass, Stephen and Vickers, Paul},
  title     = {Sonification Design and Aesthetics},
  booktitle = {The Sonification Handbook},
  editor    = {Hermann, Thomas and Hunt, Andy and Neuhoff, John G.},
  publisher = {Logos Verlag},
  address   = {Berlin},
  year      = {2011},
  pages     = {145--171},
  url       = {https://sonification.de/handbook/chapters/chapter7/}
}

@incollection{bovermann2011laboratory,
  author    = {Bovermann, Till and Rohrhuber, Julian and de Campo, Alberto},
  title     = {Laboratory Methods for Experimental Sonification},
  booktitle = {The Sonification Handbook},
  editor    = {Hermann, Thomas and Hunt, Andy and Neuhoff, John G.},
  publisher = {Logos Verlag},
  address   = {Berlin},
  year      = {2011},
  pages     = {237--272},
  url       = {https://sonification.de/handbook/chapters/chapter10/}
}

@incollection{dombois2011audification,
  author    = {Dombois, Florian and Eckel, Gerhard},
  title     = {Audification},
  booktitle = {The Sonification Handbook},
  editor    = {Hermann, Thomas and Hunt, Andy and Neuhoff, John G.},
  publisher = {Logos Verlag},
  address   = {Berlin},
  year      = {2011},
  pages     = {301--324},
  url       = {https://sonification.de/handbook/chapters/chapter12/}
}

@incollection{grond2011parameter,
  author    = {Grond, Florian and Berger, Jonathan},
  title     = {Parameter Mapping Sonification},
  booktitle = {The Sonification Handbook},
  editor    = {Hermann, Thomas and Hunt, Andy and Neuhoff, John G.},
  publisher = {Logos Verlag},
  address   = {Berlin},
  year      = {2011},
  pages     = {363--397},
  url       = {https://sonification.de/handbook/chapters/chapter15/}
}

@incollection{hermann2011model,
  author    = {Hermann, Thomas},
  title     = {Model-Based Sonification},
  booktitle = {The Sonification Handbook},
  editor    = {Hermann, Thomas and Hunt, Andy and Neuhoff, John G.},
  publisher = {Logos Verlag},
  address   = {Berlin},
  year      = {2011},
  pages     = {399--427},
  url       = {https://sonification.de/handbook/chapters/chapter16/}
}

@incollection{vickers2011process,
  author    = {Vickers, Paul},
  title     = {Sonification for Process Monitoring},
  booktitle = {The Sonification Handbook},
  editor    = {Hermann, Thomas and Hunt, Andy and Neuhoff, John G.},
  publisher = {Logos Verlag},
  address   = {Berlin},
  year      = {2011},
  pages     = {455--491},
  url       = {https://sonification.de/handbook/chapters/chapter18/}
}

@phdthesis{hermann2002sonification,
  author = {Hermann, Thomas},
  title  = {Sonification for Exploratory Data Analysis},
  school = {Faculty of Technology, Bielefeld University},
  year   = {2002},
  url    = {http://sonification.de/publications/Hermann2002-SFE}
}

@inproceedings{hermann1999listen,
  author    = {Hermann, Thomas and Ritter, Helge},
  title     = {Listen to your Data: Model-Based Sonification for Data Analysis},
  booktitle = {Advances in Intelligent Computing and Multimedia Systems},
  editor    = {Lasker, G. E.},
  publisher = {International Institute for Advanced Studies in System Research and Cybernetics},
  address   = {Baden-Baden, Germany},
  year      = {1999},
  pages     = {189--194},
  note      = {Not retrieved this session; bibliographic data from Hermann \& Ritter (2002) reference list}
}

@inproceedings{hermann2002crystallization,
  author    = {Hermann, Thomas and Ritter, Helge},
  title     = {Crystallization Sonification of High-Dimensional Datasets},
  booktitle = {Proceedings of the 2002 International Conference on Auditory Display (ICAD 2002)},
  address   = {Kyoto, Japan},
  year      = {2002},
  url       = {https://www.icad.org/Proceedings/2002/HermannRitter2002.pdf}
}

@article{hermann2004sound,
  author  = {Hermann, Thomas and Ritter, Helge},
  title   = {Sound and Meaning in Auditory Data Display},
  journal = {Proceedings of the IEEE},
  volume  = {92},
  number  = {4},
  pages   = {730--741},
  year    = {2004},
  doi     = {10.1109/JPROC.2004.825904}
}

@inproceedings{hermann2004neural,
  author    = {Hermann, Thomas and Ritter, Helge},
  title     = {Neural Gas Sonification -- Growing Adaptive Interfaces for Interacting with Data},
  booktitle = {Proceedings of the Eighth International Conference on Information Visualisation (IV 2004)},
  pages     = {871--878},
  year      = {2004},
  doi       = {10.1109/IV.2004.1320243}
}

@inproceedings{hunt2004interacting,
  author    = {Hunt, Andy and Hermann, Thomas and Pauletto, Sandra},
  title     = {Interacting with Sonification Systems: Closing the Loop},
  booktitle = {Proceedings of the Eighth International Conference on Information Visualisation (IV 2004)},
  pages     = {879--884},
  year      = {2004},
  doi       = {10.1109/IV.2004.1320244}
}

@article{hermann2005introduction,
  author  = {Hermann, Thomas and Hunt, Andy},
  title   = {Guest Editors' Introduction: An Introduction to Interactive Sonification},
  journal = {IEEE MultiMedia},
  volume  = {12},
  number  = {2},
  pages   = {20--24},
  year    = {2005},
  doi     = {10.1109/MMUL.2005.26}
}

@article{hermann2005revisited,
  author  = {Hermann, Thomas and Ritter, Helge},
  title   = {Model-Based Sonification Revisited---Authors' Comments on {Hermann and Ritter}, {ICAD} 2002},
  journal = {ACM Transactions on Applied Perception},
  volume  = {2},
  number  = {4},
  pages   = {559--563},
  year    = {2005},
  doi     = {10.1145/1101530.1101557}
}

@article{enge2023unified,
  author  = {Enge, Kajetan and Rind, Alexander and Iber, Michael and H{\"o}ldrich, Robert and Aigner, Wolfgang},
  title   = {Towards a Unified Terminology for Sonification and Visualization},
  journal = {Personal and Ubiquitous Computing},
  volume  = {27},
  number  = {5},
  pages   = {1949--1963},
  year    = {2023},
  doi     = {10.1007/s00779-023-01720-5}
}

@inproceedings{enge2021time,
  author    = {Enge, Kajetan and Rind, Alexander and Iber, Michael and H{\"o}ldrich, Robert and Aigner, Wolfgang},
  title     = {It's about Time: Adopting Theoretical Constructs from Visualization for Sonification},
  booktitle = {Proceedings of Audio Mostly 2021},
  pages     = {64--71},
  year      = {2021},
  doi       = {10.1145/3478384.3478415}
}

@article{enge2024star,
  author  = {Enge, Kajetan and Elmquist, Elias and Caiola, Valentina and R{\"o}nnberg, Niklas and Rind, Alexander and Iber, Michael and others},
  title   = {Open Your Ears and Take a Look: A State-of-the-Art Report on the Integration of Sonification and Visualization},
  journal = {Computer Graphics Forum},
  volume  = {43},
  number  = {3},
  year    = {2024},
  doi     = {10.1111/cgf.15114},
  note    = {EuroVis 2024 STAR; also arXiv:2402.16558}
}

@article{elmquist2024parallel,
  author  = {Elmquist, Elias and Enge, Kajetan and Rind, Alexander and Navarra, Carlo and H{\"o}ldrich, Robert and Iber, Michael and others},
  title   = {Parallel Chords: An Audio-Visual Analytics Design for Parallel Coordinates},
  journal = {Personal and Ubiquitous Computing},
  volume  = {28},
  number  = {5},
  pages   = {657--676},
  year    = {2024},
  doi     = {10.1007/s00779-024-01795-8}
}

@article{walker2002magnitude,
  author  = {Walker, Bruce N.},
  title   = {Magnitude Estimation of Conceptual Data Dimensions for Use in Sonification},
  journal = {Journal of Experimental Psychology: Applied},
  volume  = {8},
  number  = {4},
  pages   = {211--221},
  year    = {2002},
  doi     = {10.1037/1076-898X.8.4.211}
}

@article{walker2007consistency,
  author  = {Walker, Bruce N.},
  title   = {Consistency of Magnitude Estimations with Conceptual Data Dimensions Used for Sonification},
  journal = {Applied Cognitive Psychology},
  volume  = {21},
  number  = {5},
  pages   = {579--599},
  year    = {2007},
  doi     = {10.1002/acp.1291}
}

@article{walker2005mappings,
  author  = {Walker, Bruce N. and Kramer, Gregory},
  title   = {Mappings and Metaphors in Auditory Displays: An Experimental Assessment},
  journal = {ACM Transactions on Applied Perception},
  volume  = {2},
  number  = {4},
  pages   = {407--412},
  year    = {2005},
  doi     = {10.1145/1101530.1101534},
  note    = {Reprint of ICAD 1996 paper with appendix of results}
}

@article{walker2010universal,
  author  = {Walker, Bruce N. and Mauney, Lisa M.},
  title   = {Universal Design of Auditory Graphs: A Comparison of Sonification Mappings for Visually Impaired and Sighted Listeners},
  journal = {ACM Transactions on Accessible Computing},
  volume  = {2},
  number  = {3},
  pages   = {12:1--12:16},
  year    = {2010},
  doi     = {10.1145/1714458.1714459}
}

@article{smith2005effects,
  author  = {Smith, Daniel R. and Walker, Bruce N.},
  title   = {Effects of Auditory Context Cues and Training on Performance of a Point Estimation Sonification Task},
  journal = {Applied Cognitive Psychology},
  volume  = {19},
  number  = {8},
  pages   = {1065--1087},
  year    = {2005},
  doi     = {10.1002/acp.1146}
}

@inproceedings{nees2007listener,
  author    = {Nees, Michael A. and Walker, Bruce N.},
  title     = {Listener, Task, and Auditory Graph: Toward a Conceptual Model of Auditory Graph Comprehension},
  booktitle = {Proceedings of the 13th International Conference on Auditory Display (ICAD 2007)},
  address   = {Montr{\'e}al, Canada},
  pages     = {266--273},
  year      = {2007},
  url       = {https://sonify.psych.gatech.edu/publications/pdfs/2007ICAD-NeesWalker.pdf},
  note      = {Page range inferred from proceedings page header ICAD-269; verify}
}

@article{nees2008density,
  author  = {Nees, Michael A. and Walker, Bruce N.},
  title   = {Data Density and Trend Reversals in Auditory Graphs: Effects on Point-Estimation and Trend-Identification Tasks},
  journal = {ACM Transactions on Applied Perception},
  volume  = {5},
  number  = {3},
  pages   = {13:1--13:24},
  year    = {2008},
  doi     = {10.1145/1402236.1402237}
}

@inproceedings{nees2019eight,
  author    = {Nees, Michael A.},
  title     = {Eight Components of a Design Theory of Sonification},
  booktitle = {Proceedings of the 25th International Conference on Auditory Display (ICAD 2019)},
  pages     = {176--183},
  year      = {2019},
  doi       = {10.21785/icad2019.048}
}

@misc{nees2018killer,
  author       = {Nees, Michael A.},
  title        = {Auditory Graphs Are Not the ``Killer App'' of Sonification, But They Work},
  howpublished = {PsyArXiv preprint},
  year         = {2018},
  doi          = {10.31234/osf.io/b5akd},
  note         = {Preprint}
}

@inproceedings{neuhoff2019doomed,
  author    = {Neuhoff, John G.},
  title     = {Is Sonification Doomed to Fail?},
  booktitle = {Proceedings of the 25th International Conference on Auditory Display (ICAD 2019)},
  pages     = {327--330},
  year      = {2019},
  doi       = {10.21785/icad2019.069}
}

@article{neuhoff2002pitch,
  author  = {Neuhoff, John G. and Kramer, Gregory and Wayand, Joseph},
  title   = {Pitch and Loudness Interact in Auditory Displays: Can the Data Get Lost in the Map?},
  journal = {Journal of Experimental Psychology: Applied},
  volume  = {8},
  number  = {1},
  pages   = {17--25},
  year    = {2002},
  doi     = {10.1037/1076-898X.8.1.17}
}

@article{dubus2013systematic,
  author  = {Dubus, Ga{\"e}l and Bresin, Roberto},
  title   = {A Systematic Review of Mapping Strategies for the Sonification of Physical Quantities},
  journal = {PLoS ONE},
  volume  = {8},
  number  = {12},
  pages   = {e82491},
  year    = {2013},
  doi     = {10.1371/journal.pone.0082491}
}

@inproceedings{ferguson2018perceptual,
  author    = {Ferguson, Jamie and Brewster, Stephen A.},
  title     = {Investigating Perceptual Congruence between Data and Display Dimensions in Sonification},
  booktitle = {Proceedings of the 2018 CHI Conference on Human Factors in Computing Systems},
  pages     = {1--9},
  year      = {2018},
  doi       = {10.1145/3173574.3174185}
}

@inproceedings{ferguson2017psychoacoustic,
  author    = {Ferguson, Jamie and Brewster, Stephen A.},
  title     = {Evaluation of Psychoacoustic Sound Parameters for Sonification},
  booktitle = {Proceedings of the 19th ACM International Conference on Multimodal Interaction (ICMI '17)},
  pages     = {120--127},
  year      = {2017},
  doi       = {10.1145/3136755.3136783}
}

@inproceedings{fitzpatrick2023frequency,
  author    = {Fitzpatrick, Joe and Neff, Flaithri},
  title     = {Identifying a Perceptually Congruent Frequency Range for Auditory Line Charts},
  booktitle = {Proceedings of the 28th International Conference on Auditory Display (ICAD 2023)},
  pages     = {1--7},
  year      = {2023},
  doi       = {10.21785/icad2023.1904}
}

@article{wang2022seeing,
  author  = {Wang, R. and Jung, C. and Kim, Y.},
  title   = {Seeing Through Sounds: Mapping Auditory Dimensions to Data and Charts for People with Visual Impairments},
  journal = {Computer Graphics Forum},
  volume  = {41},
  number  = {3},
  pages   = {71--83},
  year    = {2022},
  doi     = {10.1111/cgf.14523}
}

@article{stevens1957psychophysical,
  author  = {Stevens, S. S.},
  title   = {On the Psychophysical Law},
  journal = {Psychological Review},
  volume  = {64},
  number  = {3},
  pages   = {153--181},
  year    = {1957},
  doi     = {10.1037/h0046162}
}

@article{stevens1937scale,
  author  = {Stevens, S. S. and Volkmann, J. and Newman, E. B.},
  title   = {A Scale for the Measurement of the Psychological Magnitude Pitch},
  journal = {The Journal of the Acoustical Society of America},
  volume  = {8},
  number  = {3},
  pages   = {185--190},
  year    = {1937},
  doi     = {10.1121/1.1915893}
}

@book{oshaughnessy1987speech,
  author    = {O'Shaughnessy, Douglas},
  title     = {Speech Communication: Human and Machine},
  publisher = {Addison-Wesley},
  address   = {Reading, MA},
  year      = {1987},
  note      = {Commonly cited source of the mel formula $m = 2595\log_{10}(1+f/700)$; not retrieved this session}
}

@article{zwicker1961subdivision,
  author  = {Zwicker, E.},
  title   = {Subdivision of the Audible Frequency Range into Critical Bands (Frequenzgruppen)},
  journal = {The Journal of the Acoustical Society of America},
  volume  = {33},
  number  = {2},
  pages   = {248},
  year    = {1961},
  doi     = {10.1121/1.1908630}
}

@article{traunmuller1990analytical,
  author  = {Traunm{\"u}ller, Hartmut},
  title   = {Analytical Expressions for the Tonotopic Sensory Scale},
  journal = {The Journal of the Acoustical Society of America},
  volume  = {88},
  number  = {1},
  pages   = {97--100},
  year    = {1990},
  doi     = {10.1121/1.399849}
}

@article{glasberg1990derivation,
  author  = {Glasberg, Brian R. and Moore, Brian C. J.},
  title   = {Derivation of Auditory Filter Shapes from Notched-Noise Data},
  journal = {Hearing Research},
  volume  = {47},
  number  = {1--2},
  pages   = {103--138},
  year    = {1990},
  doi     = {10.1016/0378-5955(90)90170-T}
}

@book{fastl2007psychoacoustics,
  author    = {Fastl, Hugo and Zwicker, Eberhard},
  title     = {Psychoacoustics: Facts and Models},
  edition   = {3rd},
  publisher = {Springer},
  address   = {Berlin, Heidelberg},
  year      = {2007},
  doi       = {10.1007/978-3-540-68888-4}
}

@techreport{iso226_2023,
  author      = {{International Organization for Standardization}},
  title       = {{ISO} 226:2023 Acoustics -- Normal Equal-Loudness-Level Contours},
  institution = {ISO},
  address     = {Geneva},
  type        = {International Standard},
  number      = {ISO 226:2023 (Edition 3)},
  year        = {2023},
  url         = {https://www.iso.org/standard/83117.html}
}

@article{suzuki2004equal,
  author  = {Suzuki, Yôiti and Takeshima, Hisashi},
  title   = {Equal-Loudness-Level Contours for Pure Tones},
  journal = {The Journal of the Acoustical Society of America},
  volume  = {116},
  number  = {2},
  pages   = {918--933},
  year    = {2004},
  doi     = {10.1121/1.1763601}
}

@article{fletcher1933loudness,
  author  = {Fletcher, Harvey and Munson, W. A.},
  title   = {Loudness, Its Definition, Measurement and Calculation},
  journal = {The Journal of the Acoustical Society of America},
  volume  = {5},
  number  = {2},
  pages   = {82--108},
  year    = {1933},
  doi     = {10.1121/1.1915637}
}

@article{wier1977frequency,
  author  = {Wier, Craig C. and Jesteadt, Walt and Green, David M.},
  title   = {Frequency Discrimination as a Function of Frequency and Sensation Level},
  journal = {The Journal of the Acoustical Society of America},
  volume  = {61},
  number  = {1},
  pages   = {178--184},
  year    = {1977},
  doi     = {10.1121/1.381251}
}

@article{jesteadt1977intensity,
  author  = {Jesteadt, Walt and Wier, Craig C. and Green, David M.},
  title   = {Intensity Discrimination as a Function of Frequency and Sensation Level},
  journal = {The Journal of the Acoustical Society of America},
  volume  = {61},
  number  = {1},
  pages   = {169--177},
  year    = {1977},
  doi     = {10.1121/1.381278}
}

@article{abel1972duration,
  author  = {Abel, Sharon M.},
  title   = {Duration Discrimination of Noise and Tone Bursts},
  journal = {The Journal of the Acoustical Society of America},
  volume  = {51},
  number  = {4B},
  pages   = {1219--1223},
  year    = {1972},
  doi     = {10.1121/1.1912963}
}

@article{drake1993tempo,
  author  = {Drake, Carolyn and Botte, Marie-Claire},
  title   = {Tempo Sensitivity in Auditory Sequences: Evidence for a Multiple-Look Model},
  journal = {Perception \& Psychophysics},
  volume  = {54},
  number  = {3},
  pages   = {277--286},
  year    = {1993},
  doi     = {10.3758/BF03205262}
}

@article{hirsh1959temporal,
  author  = {Hirsh, Ira J.},
  title   = {Auditory Perception of Temporal Order},
  journal = {The Journal of the Acoustical Society of America},
  volume  = {31},
  number  = {6},
  pages   = {759--767},
  year    = {1959},
  doi     = {10.1121/1.1907782}
}

@book{london2012hearing,
  author    = {London, Justin},
  title     = {Hearing in Time: Psychological Aspects of Musical Meter},
  edition   = {2nd},
  publisher = {Oxford University Press},
  year      = {2012},
  doi       = {10.1093/acprof:oso/9780199744374.001.0001}
}

@article{repp2005sensorimotor,
  author  = {Repp, Bruno H.},
  title   = {Sensorimotor Synchronization: A Review of the Tapping Literature},
  journal = {Psychonomic Bulletin \& Review},
  volume  = {12},
  number  = {6},
  pages   = {969--992},
  year    = {2005},
  doi     = {10.3758/BF03206433}
}

@book{bregman1990auditory,
  author    = {Bregman, Albert S.},
  title     = {Auditory Scene Analysis: The Perceptual Organization of Sound},
  publisher = {MIT Press},
  address   = {Cambridge, MA},
  year      = {1990},
  doi       = {10.7551/mitpress/1486.001.0001}
}

@article{moore2012properties,
  author  = {Moore, Brian C. J. and Gockel, Hedwig E.},
  title   = {Properties of Auditory Stream Formation},
  journal = {Philosophical Transactions of the Royal Society B: Biological Sciences},
  volume  = {367},
  number  = {1591},
  pages   = {919--931},
  year    = {2012},
  doi     = {10.1098/rstb.2011.0355}
}

@article{melara1990interaction,
  author  = {Melara, Robert D. and Marks, Lawrence E.},
  title   = {Interaction among Auditory Dimensions: Timbre, Pitch, and Loudness},
  journal = {Perception \& Psychophysics},
  volume  = {48},
  number  = {2},
  pages   = {169--178},
  year    = {1990},
  doi     = {10.3758/BF03207084}
}

@article{krumhansl1992perceptual,
  author  = {Krumhansl, Carol L. and Iverson, Paul},
  title   = {Perceptual Interactions between Musical Pitch and Timbre},
  journal = {Journal of Experimental Psychology: Human Perception and Performance},
  volume  = {18},
  number  = {3},
  pages   = {739--751},
  year    = {1992},
  doi     = {10.1037/0096-1523.18.3.739}
}

@article{plomp1965tonal,
  author  = {Plomp, R. and Levelt, W. J. M.},
  title   = {Tonal Consonance and Critical Bandwidth},
  journal = {The Journal of the Acoustical Society of America},
  volume  = {38},
  number  = {4},
  pages   = {548--560},
  year    = {1965},
  doi     = {10.1121/1.1909741}
}

@article{shepard1964circularity,
  author  = {Shepard, Roger N.},
  title   = {Circularity in Judgments of Relative Pitch},
  journal = {The Journal of the Acoustical Society of America},
  volume  = {36},
  number  = {12},
  pages   = {2346--2353},
  year    = {1964},
  doi     = {10.1121/1.1919362}
}

@article{pollack1954information,
  author  = {Pollack, Irwin and Ficks, Lawrence},
  title   = {Information of Elementary Multidimensional Auditory Displays},
  journal = {The Journal of the Acoustical Society of America},
  volume  = {26},
  number  = {2},
  pages   = {155--158},
  year    = {1954},
  doi     = {10.1121/1.1907300}
}

@article{mills1958minimum,
  author  = {Mills, A. W.},
  title   = {On the Minimum Audible Angle},
  journal = {The Journal of the Acoustical Society of America},
  volume  = {30},
  number  = {4},
  pages   = {237--246},
  year    = {1958},
  doi     = {10.1121/1.1909553}
}

@article{middlebrooks1991sound,
  author  = {Middlebrooks, John C. and Green, David M.},
  title   = {Sound Localization by Human Listeners},
  journal = {Annual Review of Psychology},
  volume  = {42},
  pages   = {135--159},
  year    = {1991},
  doi     = {10.1146/annurev.ps.42.020191.001031}
}

@article{wenzel1993localization,
  author  = {Wenzel, Elizabeth M. and Arruda, Marianne and Kistler, Doris J. and Wightman, Frederic L.},
  title   = {Localization Using Nonindividualized Head-Related Transfer Functions},
  journal = {The Journal of the Acoustical Society of America},
  volume  = {94},
  number  = {1},
  pages   = {111--123},
  year    = {1993},
  doi     = {10.1121/1.407089}
}

@article{edworthy1991improving,
  author  = {Edworthy, Judy and Loxley, Sarah and Dennis, Ian},
  title   = {Improving Auditory Warning Design: Relationship between Warning Sound Parameters and Perceived Urgency},
  journal = {Human Factors},
  volume  = {33},
  number  = {2},
  pages   = {205--231},
  year    = {1991},
  doi     = {10.1177/001872089103300206}
}

@article{gaver1986auditory,
  author  = {Gaver, William W.},
  title   = {Auditory Icons: Using Sound in Computer Interfaces},
  journal = {Human--Computer Interaction},
  volume  = {2},
  number  = {2},
  pages   = {167--177},
  year    = {1986},
  doi     = {10.1207/s15327051hci0202_3}
}

@article{gaver1993world,
  author  = {Gaver, William W.},
  title   = {What in the World Do We Hear? An Ecological Approach to Auditory Event Perception},
  journal = {Ecological Psychology},
  volume  = {5},
  number  = {1},
  pages   = {1--29},
  year    = {1993},
  doi     = {10.1207/s15326969eco0501_1}
}

@inproceedings{gaver1991effective,
  author    = {Gaver, William W. and Smith, Randall B. and O'Shea, Tim},
  title     = {Effective Sounds in Complex Systems: The {ARKola} Simulation},
  booktitle = {Proceedings of the SIGCHI Conference on Human Factors in Computing Systems (CHI '91)},
  pages     = {85--90},
  year      = {1991},
  doi       = {10.1145/108844.108857}
}

@article{blattner1989earcons,
  author  = {Blattner, Meera M. and Sumikawa, Denise A. and Greenberg, Robert M.},
  title   = {Earcons and Icons: Their Structure and Common Design Principles},
  journal = {Human--Computer Interaction},
  volume  = {4},
  number  = {1},
  pages   = {11--44},
  year    = {1989},
  doi     = {10.1207/s15327051hci0401_1}
}

@inproceedings{brewster1993evaluation,
  author    = {Brewster, Stephen A. and Wright, Peter C. and Edwards, Alistair D. N.},
  title     = {An Evaluation of Earcons for Use in Auditory Human-Computer Interfaces},
  booktitle = {Proceedings of the INTERACT '93 and CHI '93 Conference on Human Factors in Computing Systems},
  pages     = {222--227},
  year      = {1993},
  doi       = {10.1145/169059.169179}
}

@article{mcgookin2004understanding,
  author  = {McGookin, David K. and Brewster, Stephen A.},
  title   = {Understanding Concurrent Earcons: Applying Auditory Scene Analysis Principles to Concurrent Earcon Recognition},
  journal = {ACM Transactions on Applied Perception},
  volume  = {1},
  number  = {2},
  pages   = {130--155},
  year    = {2004},
  doi     = {10.1145/1024083.1024087}
}

@article{walker2013spearcons,
  author  = {Walker, Bruce N. and Lindsay, Jeffrey and Nance, Amanda and Nakano, Yoko and Palladino, Dianne K. and Dingler, Tilman and Jeon, Myounghoon},
  title   = {Spearcons (Speech-Based Earcons) Improve Navigation Performance in Advanced Auditory Menus},
  journal = {Human Factors},
  volume  = {55},
  number  = {1},
  pages   = {157--182},
  year    = {2013},
  doi     = {10.1177/0018720812450587}
}

@article{mansur1985sound,
  author  = {Mansur, Douglass L. and Blattner, Meera M. and Joy, Kenneth I.},
  title   = {Sound Graphs: A Numerical Data Analysis Method for the Blind},
  journal = {Journal of Medical Systems},
  volume  = {9},
  number  = {3},
  pages   = {163--174},
  year    = {1985},
  doi     = {10.1007/BF00996201}
}

@article{flowers1997crossmodal,
  author  = {Flowers, John H. and Buhman, Dion C. and Turnage, Kimberly D.},
  title   = {Cross-Modal Equivalence of Visual and Auditory Scatterplots for Exploring Bivariate Data Samples},
  journal = {Human Factors},
  volume  = {39},
  number  = {3},
  pages   = {341--351},
  year    = {1997},
  doi     = {10.1518/001872097778827151}
}

@article{speeth1961seismometer,
  author  = {Speeth, Sheridan Dauster},
  title   = {Seismometer Sounds},
  journal = {The Journal of the Acoustical Society of America},
  volume  = {33},
  number  = {7},
  pages   = {909--916},
  year    = {1961},
  doi     = {10.1121/1.1908843}
}

@inproceedings{dombois2001using,
  author    = {Dombois, Florian},
  title     = {Using Audification in Planetary Seismology},
  booktitle = {Proceedings of the 2001 International Conference on Auditory Display (ICAD 2001)},
  address   = {Espoo, Finland},
  year      = {2001},
  url       = {https://www.icad.org/Proceedings/2001/Dombois2001.pdf}
}

@article{pate2017auditory,
  author  = {Pat{\'e}, Arthur and Boschi, Lapo and Dubois, Dani{\`e}le and Le Carrou, Jean-Lo{\"i}c and Holtzman, Benjamin},
  title   = {Auditory Display of Seismic Data: On the Use of Experts' Categorizations and Verbal Descriptions as Heuristics for Geoscience},
  journal = {The Journal of the Acoustical Society of America},
  volume  = {141},
  number  = {3},
  pages   = {2143--2162},
  year    = {2017},
  doi     = {10.1121/1.4978441}
}

@article{kilb2012listen,
  author  = {Kilb, Debi and Peng, Zhigang and Simpson, David and Michael, Andrew and Fisher, Meghan and Rohrlick, Daniel},
  title   = {Listen, Watch, Learn: {SeisSound} Video Products},
  journal = {Seismological Research Letters},
  volume  = {83},
  number  = {2},
  pages   = {281--286},
  year    = {2012},
  doi     = {10.1785/gssrl.83.2.281}
}

@article{williamson2005sonification,
  author  = {Williamson, John and Murray-Smith, Roderick},
  title   = {Sonification of Probabilistic Feedback through Granular Synthesis},
  journal = {IEEE MultiMedia},
  volume  = {12},
  number  = {2},
  pages   = {45--52},
  year    = {2005},
  doi     = {10.1109/MMUL.2005.37}
}

@article{helmuth2024pulsars,
  author  = {Helmuth, Mara},
  title   = {Letting Pulsars Sing: Sonification with Granular Synthesis},
  journal = {Journal of the Audio Engineering Society},
  volume  = {72},
  number  = {5},
  pages   = {352--359},
  year    = {2024},
  doi     = {10.17743/jaes.2022.0147}
}

@article{roddy2026generative,
  author  = {Roddy, Stephen and Lavelle, Aonghus and Ghosh, Subrata and Dempsey, Gene and Fanning, Liam},
  title   = {Generative Sonification of Synthetic Virology Data with Waveshaping and Granular Synthesis Techniques},
  journal = {Leonardo},
  volume  = {59},
  number  = {1},
  pages   = {40--46},
  year    = {2026},
  doi     = {10.1162/leon.a.2583}
}

@article{hildebrandt2016continuous,
  author  = {Hildebrandt, Tobias and Hermann, Thomas and Rinderle-Ma, Stefanie},
  title   = {Continuous Sonification Enhances Adequacy of Interactions in Peripheral Process Monitoring},
  journal = {International Journal of Human-Computer Studies},
  volume  = {95},
  pages   = {54--65},
  year    = {2016},
  doi     = {10.1016/j.ijhcs.2016.06.002}
}

@article{debashi2018sonification,
  author  = {Debashi, Mohamed and Vickers, Paul},
  title   = {Sonification of Network Traffic Flow for Monitoring and Situational Awareness},
  journal = {PLoS ONE},
  volume  = {13},
  number  = {4},
  pages   = {e0195948},
  year    = {2018},
  doi     = {10.1371/journal.pone.0195948}
}

@article{watson2004sonification,
  author  = {Watson, Marcus and Sanderson, Penelope},
  title   = {Sonification Supports Eyes-Free Respiratory Monitoring and Task Time-Sharing},
  journal = {Human Factors},
  volume  = {46},
  number  = {3},
  pages   = {497--517},
  year    = {2004},
  doi     = {10.1518/hfes.46.3.497.50401}
}

@article{sanderson2005advanced,
  author  = {Sanderson, Penelope M. and Watson, Marcus O. and Russell, W. John},
  title   = {Advanced Patient Monitoring Displays: Tools for Continuous Informing},
  journal = {Anesthesia \& Analgesia},
  volume  = {101},
  number  = {1},
  pages   = {161--168},
  year    = {2005},
  doi     = {10.1213/01.ANE.0000154080.67496.AE}
}

@article{anderson2009sonification,
  author  = {Anderson, Janet E. and Sanderson, Penelope},
  title   = {Sonification Design for Complex Work Domains: Dimensions and Distractors},
  journal = {Journal of Experimental Psychology: Applied},
  volume  = {15},
  number  = {3},
  pages   = {183--198},
  year    = {2009},
  doi     = {10.1037/a0016329}
}

@inproceedings{mynatt1998designing,
  author    = {Mynatt, Elizabeth D. and Back, Maribeth and Want, Roy and Baer, Michael and Ellis, Jason B.},
  title     = {Designing Audio Aura},
  booktitle = {Proceedings of the SIGCHI Conference on Human Factors in Computing Systems (CHI '98)},
  pages     = {566--573},
  year      = {1998},
  doi       = {10.1145/274644.274720}
}

@article{pauletto2009interactive,
  author  = {Pauletto, Sandra and Hunt, Andy},
  title   = {Interactive Sonification of Complex Data},
  journal = {International Journal of Human-Computer Studies},
  volume  = {67},
  number  = {11},
  pages   = {923--933},
  year    = {2009},
  doi     = {10.1016/j.ijhcs.2009.05.006}
}

@article{grond2014interactive,
  author  = {Grond, Florian and Hermann, Thomas},
  title   = {Interactive Sonification for Data Exploration: How Listening Modes and Display Purposes Define Design Guidelines},
  journal = {Organised Sound},
  volume  = {19},
  number  = {1},
  pages   = {41--51},
  year    = {2014},
  doi     = {10.1017/S1355771813000393}
}

@article{ziemer2019psychoacoustic,
  author  = {Ziemer, Tim and Schultheis, Holger},
  title   = {Psychoacoustic Auditory Display for Navigation: An Auditory Assistance System for Spatial Orientation Tasks},
  journal = {Journal on Multimodal User Interfaces},
  volume  = {13},
  number  = {3},
  pages   = {205--218},
  year    = {2019},
  doi     = {10.1007/s12193-018-0282-2}
}

@inproceedings{schwarz2019psychoacoustic,
  author    = {Schwarz, Sebastian and Ziemer, Tim},
  title     = {A Psychoacoustic Sound Design for Pulse Oximetry},
  booktitle = {Proceedings of the 25th International Conference on Auditory Display (ICAD 2019)},
  pages     = {214--221},
  year      = {2019},
  doi       = {10.21785/icad2019.024}
}

@inproceedings{ziemer2023three,
  author    = {Ziemer, Tim},
  title     = {Three-Dimensional Sonification for Image-Guided Surgery},
  booktitle = {Proceedings of the 28th International Conference on Auditory Display (ICAD 2023)},
  pages     = {8--14},
  year      = {2023},
  doi       = {10.21785/icad2023.2324}
}

@inproceedings{ziemer2024curat,
  author    = {Ziemer, Tim},
  title     = {Results from the {CURAT} Sonification Game},
  booktitle = {Proceedings of the 29th International Conference on Auditory Display (ICAD 2024)},
  pages     = {42--49},
  year      = {2024},
  doi       = {10.21785/icad2024.007}
}

@article{supper2014sublime,
  author  = {Supper, Alexandra},
  title   = {Sublime Frequencies: The Construction of Sublime Listening Experiences in the Sonification of Scientific Data},
  journal = {Social Studies of Science},
  volume  = {44},
  number  = {1},
  pages   = {34--58},
  year    = {2014},
  doi     = {10.1177/0306312713496875}
}

@article{barrass2012aesthetic,
  author  = {Barrass, Stephen},
  title   = {The Aesthetic Turn in Sonification towards a Social and Cultural Medium},
  journal = {AI \& Society},
  volume  = {27},
  number  = {2},
  pages   = {177--181},
  year    = {2012},
  doi     = {10.1007/s00146-011-0335-5}
}

@incollection{vickers2017aesthetics,
  author    = {Vickers, Paul and Hogg, Bennett and Worrall, David},
  title     = {Aesthetics of Sonification},
  booktitle = {Body, Sound and Space in Music and Beyond: Multimodal Explorations},
  editor    = {W{\"o}llner, Clemens},
  publisher = {Routledge},
  pages     = {89--109},
  year      = {2017},
  doi       = {10.4324/9781315569628-6},
  note      = {Editor name not verified via Crossref}
}

@article{roddy2014embodied,
  author  = {Roddy, Stephen and Furlong, Dermot},
  title   = {Embodied Aesthetics in Auditory Display},
  journal = {Organised Sound},
  volume  = {19},
  number  = {1},
  pages   = {70--77},
  year    = {2014},
  doi     = {10.1017/S1355771813000423}
}

@article{lenzi2020intentionality,
  author  = {Lenzi, Sara and Ciuccarelli, Paolo},
  title   = {Intentionality and Design in the Data Sonification of Social Issues},
  journal = {Big Data \& Society},
  volume  = {7},
  number  = {2},
  pages   = {2053951720944603},
  year    = {2020},
  doi     = {10.1177/2053951720944603}
}

@article{middleton2023datatomusic,
  author  = {Middleton, Jonathan and Hakulinen, Jaakko and Tiitinen, Katariina and Hella, Juho and Keskinen, Tuuli and Huuskonen, Pertti and Culver, Jeffrey and Linna, Juhani and Turunen, Markku and Ziat, Mounia and Raisamo, Roope},
  title   = {Data-to-Music Sonification and User Engagement},
  journal = {Frontiers in Big Data},
  volume  = {6},
  pages   = {1206081},
  year    = {2023},
  doi     = {10.3389/fdata.2023.1206081}
}

@inproceedings{sharif2022userfriendly,
  author    = {Sharif, Ather and Wang, Olivia H. and Muongchan, Alida T.},
  title     = {``What Makes Sonification User-Friendly?'' Exploring Usability and User-Friendliness of Sonified Responses},
  booktitle = {Proceedings of the 24th International ACM SIGACCESS Conference on Computers and Accessibility (ASSETS '22)},
  pages     = {1--5},
  year      = {2022},
  doi       = {10.1145/3517428.3550360}
}

@article{ronnberg2019musical,
  author  = {R{\"o}nnberg, Niklas},
  title   = {Musical Sonification Supports Visual Discrimination of Color Intensity},
  journal = {Behaviour \& Information Technology},
  volume  = {38},
  number  = {10},
  pages   = {1028--1037},
  year    = {2019},
  doi     = {10.1080/0144929X.2019.1657952}
}

@inproceedings{hoque2023natural,
  author    = {Hoque, Md Naimul and Ehtesham-Ul-Haque, Md and Elmqvist, Niklas and Billah, Syed Masum},
  title     = {Accessible Data Representation with Natural Sound},
  booktitle = {Proceedings of the 2023 CHI Conference on Human Factors in Computing Systems},
  pages     = {1--19},
  year      = {2023},
  doi       = {10.1145/3544548.3581087}
}

@inproceedings{siu2022audio,
  author    = {Siu, Alexa and S-H Kim, Gene and O'Modhrain, Sile and Follmer, Sean},
  title     = {Supporting Accessible Data Visualization Through Audio Data Narratives},
  booktitle = {CHI Conference on Human Factors in Computing Systems (CHI '22)},
  pages     = {1--19},
  year      = {2022},
  doi       = {10.1145/3491102.3517678}
}

@inproceedings{holloway2022infosonics,
  author    = {Holloway, Leona M. and Goncu, Cagatay and Ilsar, Alon and Butler, Matthew and Marriott, Kim},
  title     = {Infosonics: Accessible Infographics for People who are Blind using Sonification and Voice},
  booktitle = {CHI Conference on Human Factors in Computing Systems (CHI '22)},
  pages     = {1--13},
  year      = {2022},
  doi       = {10.1145/3491102.3517465}
}

@inproceedings{sharif2022voxlens,
  author    = {Sharif, Ather and Wang, Olivia H. and Muongchan, Alida T. and Reinecke, Katharina and Wobbrock, Jacob O.},
  title     = {{VoxLens}: Making Online Data Visualizations Accessible with an Interactive {JavaScript} Plug-In},
  booktitle = {CHI Conference on Human Factors in Computing Systems (CHI '22)},
  pages     = {1--19},
  year      = {2022},
  doi       = {10.1145/3491102.3517431}
}

@inproceedings{thompson2023chartreader,
  author    = {Thompson, John R. and Martinez, Jesse J. and Sarikaya, Alper and Cutrell, Edward and Lee, Bongshin},
  title     = {Chart Reader: Accessible Visualization Experiences Designed with Screen Reader Users},
  booktitle = {Proceedings of the 2023 CHI Conference on Human Factors in Computing Systems},
  pages     = {1--18},
  year      = {2023},
  doi       = {10.1145/3544548.3581186}
}

@article{zong2022rich,
  author  = {Zong, Jonathan and Lee, Crystal and Lundgard, Alan and Jang, JiWoong and Hajas, Daniel and Satyanarayan, Arvind},
  title   = {Rich Screen Reader Experiences for Accessible Data Visualization},
  journal = {Computer Graphics Forum},
  volume  = {41},
  number  = {3},
  pages   = {15--27},
  year    = {2022},
  doi     = {10.1111/cgf.14519}
}

@article{kim2021accessible,
  author  = {Kim, N. W. and Joyner, S. C. and Riegelhuth, A. and Kim, Y.},
  title   = {Accessible Visualization: Design Space, Opportunities, and Challenges},
  journal = {Computer Graphics Forum},
  volume  = {40},
  number  = {3},
  pages   = {173--188},
  year    = {2021},
  doi     = {10.1111/cgf.14298}
}

@inproceedings{kim2024erie,
  author    = {Kim, Hyeok and Kim, Yea-Seul and Hullman, Jessica},
  title     = {Erie: A Declarative Grammar for Data Sonification},
  booktitle = {Proceedings of the CHI Conference on Human Factors in Computing Systems (CHI '24)},
  pages     = {1--19},
  year      = {2024},
  doi       = {10.1145/3613904.3642442}
}

@inproceedings{zong2024umwelt,
  author    = {Zong, Jonathan and Pedraza Pineros, Isabella and Chen, Mengzhu (Katie) and Hajas, Daniel and Satyanarayan, Arvind},
  title     = {Umwelt: Accessible Structured Editing of Multi-Modal Data Representations},
  booktitle = {Proceedings of the CHI Conference on Human Factors in Computing Systems (CHI '24)},
  pages     = {1--20},
  year      = {2024},
  doi       = {10.1145/3613904.3641996}
}

@inproceedings{zhao2024speechtone,
  author    = {Zhao, Yichun and Lu, Jingyi and Nacenta, Miguel A.},
  title     = {Speech-based Mark for Data Sonification},
  booktitle = {Proceedings of the 26th International ACM SIGACCESS Conference on Computers and Accessibility (ASSETS '24)},
  pages     = {1--5},
  year      = {2024},
  doi       = {10.1145/3663548.3688514}
}

@inproceedings{kantan2024mixing,
  author    = {Kantan, Prithvi Ravi and Dahl, Sofia and Spaich, Erika G.},
  title     = {Adapting Audio Mixing Principles and Tools to Parameter Mapping Sonification Design},
  booktitle = {Proceedings of the 29th International Conference on Auditory Display (ICAD 2024)},
  pages     = {148--155},
  year      = {2024},
  doi       = {10.21785/icad2024.034}
}

@inproceedings{crozzoli2025signal,
  author    = {Crozzoli, Miguel Angel and Muruaga, Gonzalo and Magnusson, Thor},
  title     = {Signal and Symbol Representations in Sonification: Structuring Data-to-Display Transformations},
  booktitle = {Proceedings of the 30th International Conference on Auditory Display (ICAD 2025)},
  pages     = {37--44},
  year      = {2025},
  doi       = {10.21785/icad2025.094}
}

@article{seica2024design,
  author  = {Sei{\c{c}}a, Mariana and Roque, Lic{\'i}nio and Martins, Pedro and Cardoso, F. Am{\'i}lcar},
  title   = {A Design Model for Systemic Sonification},
  journal = {Journal of the Audio Engineering Society},
  volume  = {72},
  number  = {9},
  pages   = {571--584},
  year    = {2024},
  doi     = {10.17743/jaes.2022.0165}
}

@inproceedings{seica2020contrasts,
  author    = {Sei{\c{c}}a, Mariana and Roque, Lic{\'i}nio and Martins, Pedro and Cardoso, F. Am{\'i}lcar},
  title     = {Contrasts and Similarities between Two Audio Research Communities in Evaluating Auditory Artefacts},
  booktitle = {Proceedings of the 15th International Audio Mostly Conference (AM '20)},
  pages     = {183--190},
  year      = {2020},
  doi       = {10.1145/3411109.3411146}
}

@inproceedings{lindborg2025trends,
  author    = {Lindborg, Per Magnus and Caiola, Valentina and Lenzi, Sara and Ciuccarelli, Paolo},
  title     = {Trends in Goals and Authorship Over Three Decades: Insights from the Data Sonification Archive},
  booktitle = {Proceedings of the 30th International Conference on Auditory Display (ICAD 2025)},
  pages     = {176--182},
  year      = {2025},
  doi       = {10.21785/icad2025.017}
}

@book{worrall2019sonification,
  author    = {Worrall, David},
  title     = {Sonification Design: From Data to Intelligible Soundfields},
  series    = {Human--Computer Interaction Series},
  publisher = {Springer},
  address   = {Cham},
  year      = {2019},
  doi       = {10.1007/978-3-030-01497-1}
}

@inproceedings{hommel2025ai,
  author    = {Hommel, Fabian and Hermann, Thomas},
  title     = {{AI} Meets Sonification: Research Agenda and Technology Demonstration},
  booktitle = {Proceedings of the 30th International Conference on Auditory Display (ICAD 2025)},
  pages     = {56--63},
  year      = {2025},
  doi       = {10.21785/icad2025.037}
}

@inproceedings{budziszewski2025tools,
  author    = {Budziszewski, Sofia Vallejo and Gro{\ss}-Vogt, Katharina},
  title     = {Development of Sonification Tools: History and Outlook},
  booktitle = {Proceedings of the 30th International Conference on Auditory Display (ICAD 2025)},
  pages     = {45--54},
  year      = {2025},
  doi       = {10.21785/icad2025.071}
}

@inproceedings{dalziel2025harmonic,
  author    = {Dalziel, Tony and Lieck, Robert},
  title     = {Harmonic Sonification of High-Dimensional Continuous Data},
  booktitle = {Proceedings of the 30th International Conference on Auditory Display (ICAD 2025)},
  pages     = {150--157},
  year      = {2025},
  doi       = {10.21785/icad2025.019}
}

@inproceedings{born2025pyagen,
  author    = {Born, Luka Finn and Hermann, Thomas},
  title     = {Pya {AGen} -- Audio Generators for Sonification in Python},
  booktitle = {Proceedings of the 30th International Conference on Auditory Display (ICAD 2025)},
  pages     = {20--27},
  year      = {2025},
  doi       = {10.21785/icad2025.021}
}

@inproceedings{muller2025quantum,
  author    = {M{\"u}ller, Jannis L. and Freye, Arthur and Hermann, Thomas},
  title     = {Interactive Sonification of {2D} Quantum Systems},
  booktitle = {Proceedings of the 30th International Conference on Auditory Display (ICAD 2025)},
  pages     = {141--148},
  year      = {2025},
  doi       = {10.21785/icad2025.030}
}

@inproceedings{laczko2025pixasonics,
  author    = {Laczko, Balint and Jensenius, Alexander Refsum},
  title     = {Pixasonics: An Image Sonification Toolbox for Python},
  booktitle = {Proceedings of the 30th International Conference on Auditory Display (ICAD 2025)},
  pages     = {28--35},
  year      = {2025},
  doi       = {10.21785/icad2025.074}
}

@inproceedings{reinsch2023sonecules,
  author    = {Reinsch, Dennis and Hermann, Thomas},
  title     = {Sonecules: A Python Sonification Architecture},
  booktitle = {Proceedings of the 28th International Conference on Auditory Display (ICAD 2023)},
  pages     = {62--69},
  year      = {2023},
  doi       = {10.21785/icad2023.5580}
}

@inproceedings{trayford2023strauss,
  author    = {Trayford, James W. and Harrison, Chris M.},
  title     = {Introducing {STRAUSS}: A Flexible Sonification Python Package},
  booktitle = {Proceedings of the 28th International Conference on Auditory Display (ICAD 2023)},
  pages     = {249--256},
  year      = {2023},
  doi       = {10.21785/icad2023.1978}
}

@article{trayford2025strauss,
  author  = {Trayford, James W. and Youles, Samantha and Harrison, Chris and Shepherd, Rose and Bonne, Nicolas},
  title   = {{STRAUSS}: Sonification Tools and Resources for Analysis Using Sound Synthesis},
  journal = {Journal of Open Source Software},
  volume  = {10},
  number  = {109},
  pages   = {7875},
  year    = {2025},
  doi     = {10.21105/joss.07875}
}

@inproceedings{peng2023siren,
  author    = {Peng, Tristan and Choi, Hongchan and Berger, Jonathan},
  title     = {Siren: Creative and Extensible Sonification on the Web},
  booktitle = {Proceedings of the 28th International Conference on Auditory Display (ICAD 2023)},
  pages     = {78--84},
  year      = {2023},
  doi       = {10.21785/icad2023.6167}
}

@inproceedings{poret2023ossia,
  author    = {Poret, Maxime and Celerier, Jean-Micha{\"e}l and Desainte-Catherine, Myriam and Semal, Catherine},
  title     = {Proof of Concept of a Generic Toolkit for Sonification: The Sonification Cell in Ossia Score},
  booktitle = {Proceedings of the 28th International Conference on Auditory Display (ICAD 2023)},
  pages     = {23--30},
  year      = {2023},
  doi       = {10.21785/icad2023.4039}
}

@inproceedings{cantrell2021highcharts,
  author    = {Cantrell, Stanley J. and Walker, Bruce N. and Moseng, {\O}ystein},
  title     = {Highcharts Sonification Studio: An Online, Open-Source, Extensible, and Accessible Data Sonification Tool},
  booktitle = {Proceedings of the 26th International Conference on Auditory Display (ICAD 2021)},
  pages     = {210--216},
  year      = {2021},
  doi       = {10.21785/icad2021.005}
}

@inproceedings{beadling2023spatialised,
  author    = {Beadling, Andrew and Vickers, Paul},
  title     = {Listener Perception of Spatialised Audio for Embodied Interaction in Sonification},
  booktitle = {Proceedings of the 28th International Conference on Auditory Display (ICAD 2023)},
  pages     = {15--22},
  year      = {2023},
  doi       = {10.21785/icad2023.3814}
}

@inproceedings{zelada2023unnatural,
  author    = {Zelada, Eloysa and {\c{C}}amc{\i}, An{\i}l},
  title     = {Unnatural Nature: Spatial Sonification of Climate Change Data},
  booktitle = {Proceedings of the 28th International Conference on Auditory Display (ICAD 2023)},
  pages     = {121--128},
  year      = {2023},
  doi       = {10.21785/icad2023.9190}
}

@inproceedings{linke2024somson,
  author    = {Linke, Simon and Ziemer, Tim and Desainte-Catherine, Myriam and Semal, Catherine},
  title     = {{SOMson} -- Sonification of Multidimensional Data in Kohonen Maps},
  booktitle = {Proceedings of the 29th International Conference on Auditory Display (ICAD 2024)},
  pages     = {50--57},
  year      = {2024},
  doi       = {10.21785/icad2024.008}
}

@inproceedings{elmquist2024birdsong,
  author    = {Elmquist, Elias and Ejdbo, Malin and Bock, Alexander and Thaler, David S. and Ynnerman, Anders and R{\"o}nnberg, Niklas},
  title     = {Birdsongification: Contextual and Complementary Sonification for Biology Visualization},
  booktitle = {Proceedings of the 29th International Conference on Auditory Display (ICAD 2024)},
  pages     = {34--41},
  year      = {2024},
  doi       = {10.21785/icad2024.006}
}

@inproceedings{pile2024spinwave,
  author    = {Pile, Santa and Lesota, Oleg and Peter, Silvan David and Humer, Christina and Di Gasser, Martin},
  title     = {Spin-Wave Voices: Sonification of Nanoscale Spin Waves as an Engagement and Research Tool},
  booktitle = {Proceedings of the 29th International Conference on Auditory Display (ICAD 2024)},
  pages     = {132--139},
  year      = {2024},
  doi       = {10.21785/icad2024.024}
}

@inproceedings{riber2024corot,
  author    = {Riber, Adri{\'a}n Garc{\'i}a and Serradilla, Francisco},
  title     = {{CoRoT} Light Curve Ambisonics/Binaural Sonification},
  booktitle = {Proceedings of the 29th International Conference on Auditory Display (ICAD 2024)},
  pages     = {66--72},
  year      = {2024},
  doi       = {10.21785/icad2024.010}
}

@inproceedings{riber2022tess,
  author    = {Riber, Adrian Garcia and Garcia, Francisco Serradilla},
  title     = {Sonification of {TESS} Data Validation Timeseries Files},
  booktitle = {Proceedings of the 27th International Conference on Auditory Display (ICAD 2022)},
  pages     = {69--74},
  year      = {2022},
  doi       = {10.21785/icad2022.024}
}

@inproceedings{huppenkothen2023hr,
  author    = {Huppenkothen, Daniela and Pampin, Juan and Davenport, James R. A. and Wenlock, James},
  title     = {The Sonified {Hertzsprung-Russell} Diagram},
  booktitle = {Proceedings of the 28th International Conference on Auditory Display (ICAD 2023)},
  pages     = {268--275},
  year      = {2023},
  doi       = {10.21785/icad2023.6263}
}

@inproceedings{misdariis2023audible,
  author    = {Misdariis, Nicolas and Pauletto, Sandra and Bonne, Nicolas and Harrison, Chris and Meredith, Kate and Zanella, Anita},
  title     = {The Audible Universe Workshop: An Interdisciplinary Approach to the Design and Evaluation of Tools for Astronomical Data Sonification},
  booktitle = {Proceedings of the 28th International Conference on Auditory Display (ICAD 2023)},
  pages     = {253--260},
  year      = {2023},
  doi       = {10.21785/icad2023.2124}
}

@article{zanella2022sonification,
  author  = {Zanella, A. and Harrison, C. M. and Lenzi, S. and Cooke, J. and Damsma, P. and Fleming, S. W.},
  title   = {Sonification and Sound Design for Astronomy Research, Education and Public Engagement},
  journal = {Nature Astronomy},
  volume  = {6},
  number  = {11},
  pages   = {1241--1248},
  year    = {2022},
  doi     = {10.1038/s41550-022-01721-z}
}

@article{tuckerbrown2022evaluating,
  author  = {Tucker Brown, J. and Harrison, C. M. and Zanella, A. and Trayford, J.},
  title   = {Evaluating the Efficacy of Sonification for Signal Detection in Univariate, Evenly Sampled Light Curves Using {astronify}},
  journal = {Monthly Notices of the Royal Astronomical Society},
  volume  = {516},
  number  = {4},
  pages   = {5674--5683},
  year    = {2022},
  doi     = {10.1093/mnras/stac2590}
}

@article{sawe2020using,
  author  = {Sawe, Nik and Chafe, Chris and Trevi{\~n}o, Jeffrey},
  title   = {Using Data Sonification to Overcome Science Literacy, Numeracy, and Visualization Barriers in Science Communication},
  journal = {Frontiers in Communication},
  volume  = {5},
  pages   = {46},
  year    = {2020},
  doi     = {10.3389/fcomm.2020.00046}
}

@incollection{hart1988nasatlx,
  author    = {Hart, Sandra G. and Staveland, Lowell E.},
  title     = {Development of {NASA-TLX} (Task Load Index): Results of Empirical and Theoretical Research},
  booktitle = {Human Mental Workload},
  series    = {Advances in Psychology},
  volume    = {52},
  editor    = {Hancock, Peter A. and Meshkati, Najmedin},
  publisher = {North-Holland},
  pages     = {139--183},
  year      = {1988},
  doi       = {10.1016/S0166-4115(08)62386-9}
}

@incollection{brooke1996sus,
  author    = {Brooke, John},
  title     = {{SUS}: A ``Quick and Dirty'' Usability Scale},
  booktitle = {Usability Evaluation in Industry},
  editor    = {Jordan, Patrick W. and Thomas, B. and Weerdmeester, Bernard A. and McClelland, Ian L.},
  publisher = {Taylor \& Francis},
  address   = {London},
  pages     = {189--194},
  year      = {1996}
}
```

## Errata

Dated corrections. The text above is kept as the kickoff snapshot ([research README](../README.md)).

- **2026-10-18, link check.** `http://sonification.de/publications/Hermann2002-SFE`, cited in §1 for Hermann's 2002 Bielefeld PhD thesis *Sonification for Exploratory Data Analysis*, returned HTTP 404 in CI, and sonification.de refused connections during the kickoff research. No replacement URL has been verified yet: Bielefeld University's PUB repository blocks automated requests. Re-check by hand before citing the thesis in a paper.
