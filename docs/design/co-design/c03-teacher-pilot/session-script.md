# Teacher pilot: session script

> **Status:** Draft (2026-11-01), to be checked against the shipped C03-T08 UI before use. **Length:** planned at about 90 minutes with breaks, adjusted to the teacher's needs (conformance plan §8.4). **Where:** remote video or voice call; the teacher uses their own computer, screen reader and settings.

The facilitator reads the words in quotes, or says the same in their own words. Everything else is for the facilitator. Say clearly at the start that it is SONARITHM being tested, not the teacher, and that a hard task is a finding about the software.

## Before the session (facilitator)

- [ ] The information sheet was sent in the teacher's chosen format at least two days before, and consent is recorded ([information-and-consent.md](information-and-consent.md)).
- [ ] The teacher has the build to test installed and opened once (the desktop build with C03-T08), or the facilitator has a fallback ready (screen sharing of the facilitator's machine is **not** a substitute: the point is the teacher's own screen reader).
- [ ] Note the teacher's screen reader and version, operating system, and braille display if any. These are recorded in the session record without the person's name.
- [ ] Have the results record open ([results-template.md](results-template.md)) with a fresh participant code (P1, P2, …).
- [ ] Payment details are agreed in advance, by a method the teacher can use without sighted help.

## Opening (about 10 minutes)

"Thank you for joining. Today you'll try a part of SONARITHM that plays a mathematical function as sound, so you can find its zeros, its highest and lowest points and where it has asymptotes by listening, and read the same points as a list. We're testing the software, not you: if something is hard or confusing, that's exactly what we need to hear about. You can stop, take a break or skip anything at any time, and you'll be paid for the whole session either way."

"I'll ask you to think aloud: say what you're trying, what you hear and what you expect. I won't help unless you ask, or unless you're stuck for a while; then I'll give a hint and note it."

If recording was agreed: "As agreed, I'll start the recording now. You can ask me to stop it at any time." Otherwise: "We're not recording; I'll take written notes without your name."

Ask the teacher to set their speech rate and verbosity as they normally use them.

## Tasks (about 60 minutes)

For each task: read the task, let the teacher work, and record the outcome (**unaided**, **with a hint**, **not completed**), whether the answer was correct, the time if it is easy to note, and what went wrong or surprised them. Pass criteria follow the [scripted task suite](../../accessibility-conformance-plan.md#32-scripted-task-suite): completed unaided with the keyboard and screen reader, correct answer, correct announcements, no trap.

| Task | Say | Pass criterion | Features (A11Y) |
|---|---|---|---|
| P-1 Type a function | "Please open the function input and enter f of x equals x cubed minus three x, with x from minus three to three. Then play it." (`x^3 - 3*x`, domain −3 to 3) | The function opens and its summary is heard (expression, domain, how many zeros and extrema); playback starts only on command | Function input, summary (05, 29) |
| P-2 Find the zeros | "Where does this function cross zero? Use whatever you like: listening, the list, or moving through the points." | Three zeros reported: about −1.73, 0 and 1.73 (−√3, 0, √3) | Zero earcons, critical-point list, insight navigation (23, 24, 28) |
| P-3 Find the extrema | "Where are its highest and lowest points, and what are their values?" | A local maximum at x = −1 with value 2 and a local minimum at x = 1 with value −2 | Landmark earcons, list, insights (23, 28) |
| P-4 Asymptote and gaps | "Please enter one over x minus one, with x from minus four to four. Is there anywhere the function goes off to infinity?" Then: "Now enter the square root of x, with x from minus four to four. Where is it not defined?" (`1/(x - 1)`; `sqrt(x)`) | The vertical asymptote at x = 1 is found; the undefined span from −4 to 0 is found | Asymptote and undefined earcons, list (24, 28) |
| P-5 Fix a mistake | "Please type two x plus one, written as 2x, without a multiplication sign." (`2x + 1`) | The error is noticed from the field's message, and fixed to `2*x + 1` | Live diagnostics (05, 29) |
| P-6 The audification calculator | "Please enter sine of x, with x from zero to one hundred and two thousand samples, and play it as audification. What speed-up does the calculator show, how long does it play, and in which frequency band?" Then: "Change the speed-up to one hundred." | k, the duration and the band are read with their units; changing k updates them | Calculator (05, 29; SDR-31) |
| P-7 Your own example (optional) | "Is there a function you use in class? Please try it." | Recorded as the teacher describes it; no pass criterion | Any |

If the UI in the shipped C03-T08 differs from these wordings (control names, where the list lives), keep the task's goal and change only the wording.

## Debrief (about 15 minutes)

Ask openly; record the answers as notes, without the person's name:

1. "Which of these would you use with students, and for what?"
2. "Were the sounds for zeros, asymptotes, gaps and highest or lowest points easy to tell apart? Was any of them unpleasant, too loud or too quiet?"
3. "Were the words right? For example 'local maximum', 'inflection point', 'vertical asymptote', 'undefined': would you or your students say something else?"
4. "Was anything too fast, too slow or too much to listen to?"
5. "Did the list and the sound agree? Was it clear which sound meant which point?"
6. "What is missing that you'd need in a lesson?"

## Closing (about 5 minutes)

"Thank you. Here is what happens next: I'll write up what we found, without your name, and turn it into changes to SONARITHM. If you'd like, I'll tell you what changed because of today. Your payment will be sent by [the agreed method] within [the agreed time]." Stop the recording if there was one, and confirm the payment details.

## After the session (facilitator)

- [ ] Complete the session record the same day ([results-template.md](results-template.md)): outcomes, findings, quotes paraphrased or anonymised.
- [ ] Pay the teacher as agreed.
- [ ] Store any recording outside the repository, where only the owner can reach it, and delete it when the findings are written up, unless the teacher agreed otherwise.
- [ ] After at least three sessions, write the aggregate summary and log the findings (`F-###`, from CURVASONA's range), then triage them into issues with severities (conformance plan §9).
