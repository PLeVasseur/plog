---
layout: post
title: "Trashy Commits + Interactive Rebase = Great Git History"
date: 2024-09-04
---

# Trashy Commits + Interactive Rebase = Great Git History

It feels like there's some "step function" changes I've had in productivity in the last couple of years, some of which came easier than others. I was thinking about this while watching one of [Theo](https://t3.gg/)'s videos a week or two ago in which he explained how his experience with git allowed him to fearlessly make changes in his coding assignments in school, since he'd always have the commit history on the remote.

Enabling vim motions in RustRover and then switching to Neovim (btw) was a big one, but I'll leave that one for another day. (shoutout to ThePrimeagen & TJ)

Git's interactice rebase though, is _super_ powerful, while being really easy to understand and lets you continue to commit like a mad scientist while iterating and have clean commit history.

Let's chat about how you too can make trashy commits and turn them into reasonable and parseable commit history using git's interactive rebase.

## So you're making trashy commits

If you're anything like me, you rarely are coming into developing software with all (any of?) the details figured out. You've seen if the problem you're solving has already been solved or can be slotted into some paradigm, then you're off to the races, iterating.

My git commit history often looks like I'd be better off left with a box of crayons in the corner than let write software.

At least, before I run git's interactice rebase.

## And then you run git's interactice rebase

I'm a little embarrased to admit that despite [Greg Medding](https://github.com/gregmedd)'s suggestion six months or so ago, I didn't start using interactive rebase until a month ago. I think anyone that has had to rebase a feature branch which has gone stale will associate bad vibes to git's rebase. I'm glad to report using interactice rebase is super easy.

### So what did you do if not interactice rebase?

Until I started using git's interactive rebase I was using the [following](https://stackoverflow.com/a/5201642/5145535) to squash all commits in a feature branch before merging.

Assume we're squashing the last three commits which belong to our feature branch:

```bash
git reset --soft HEAD~3
git commit
```

Which, to be clear, _does work_ and is perfectly fine if you just want to squash the commits.

If you want to do anything more complex like reorder commits, squash, or fixup you're better served by git's interactive rebase.

### So then, interactive rebase

I feel like I've only just started to scratch the surface of the power of git's interactive rebase. From hearing Greg tell it, once we figure out how to properly structure our commits it's even better.

But okay, imagine that we're a noob at it, like I am. What's the use cases here?

Let's take a few of them I've run into over the last month.

#### The squash example again

Imagine you've got some commit history like this:

```
commit 9744a6da1054a06292be16dabbddf7788144bc7e (HEAD -> article/trashy-rebase)
Author: Peter LeVasseur <plevasseur@gmail.com>
Date:   Wed Sep 4 14:13:59 2024 -0400

    Add a bit about interactive rebase use cases

commit a81eec6d6187a717b7ea5004798cbd2db86f765e
Author: Peter LeVasseur <plevasseur@gmail.com>
Date:   Wed Sep 4 14:03:58 2024 -0400

    Added example of how previously squashed commits

commit 5760b9c46fb2f9f28cf619a6424720036daad354
Author: Peter LeVasseur <plevasseur@gmail.com>
Date:   Wed Sep 4 13:54:29 2024 -0400

    Start article

commit 9328e37d8b3e063f4e01e94b78501e61e7fda033 (origin/main, origin/HEAD)
Merge: 69247df f57d138
Author: Pete LeVasseur <plevasseur@gmail.com>
Date:   Fri Aug 30 21:58:16 2024 -0400

    Merge pull request #36 from PLeVasseur/article/gracefully-cpp-interop-drop-impl.md
    
    Article/gracefully cpp interop drop impl.md
```

And you want to collapse the last three commits, just like above.

We can use git's interactive rebase instead, like this:

```
git rebase --interactive 9328e37d8b3e063f4e01e94b78501e61e7fda033
```

where we choose the commit _immediately prior_ to where we want to initiate the rebase. Perhaps in most cases the latest commit on `main` right before all your commits on the feature branch.

So what happens when we execute the above?

We get this. Which can look like a lot on first glance, but for this basic case is really simple. I do really like that the possible commands are laid out here for us to be able to reference!

```
pick 5760b9c Start article
pick a81eec6 Added example of how previously squashed commits
pick 9744a6d Add a bit about interactive rebase use cases

# Rebase 9328e37..9744a6d onto 9328e37 (3 commands)
#
# Commands:
# p, pick <commit> = use commit
# r, reword <commit> = use commit, but edit the commit message
# e, edit <commit> = use commit, but stop for amending
# s, squash <commit> = use commit, but meld into previous commit
# f, fixup [-C | -c] <commit> = like "squash" but keep only the previous
#                    commit's log message, unless -C is used, in which case
#                    keep only this commit's message; -c is same as -C but
#                    opens the editor
# x, exec <command> = run command (the rest of the line) using shell
# b, break = stop here (continue rebase later with 'git rebase --continue')
# d, drop <commit> = remove commit
# l, label <label> = label current HEAD with a name
# t, reset <label> = reset HEAD to a label
# m, merge [-C <commit> | -c <commit>] <label> [# <oneline>]
# .       create a merge commit using the original merge commit's
# .       message (or the oneline, if no original merge commit was
# .       specified); use -c <commit> to reword the commit message
#
# These lines can be re-ordered; they are executed from top to bottom.
#
# If you remove a line here THAT COMMIT WILL BE LOST.
#
# However, if you remove everything, the rebase will be aborted.
#
```

In this case, since we're going to want to just have a single commit and change-up what's written there we can issue the following commands:

```
reword 5760b9c Start article
fixup a81eec6 Added example of how previously squashed commits
fixup 9744a6d Add a bit about interactive rebase use cases
```

We are telling the interactive rebase to let us reword the first commit and then "squash" the next two commits into the first, but discarding their commit messages.

So we can then save this -- I use vim / Neovim (btw) so I hit it with the `:wq` and then see:

```
Start article

# Please enter the commit message for your changes. Lines starting
# with '#' will be ignored, and an empty message aborts the commit.
#
# Date:      Wed Sep 4 13:54:29 2024 -0400
#
# interactive rebase in progress; onto 9328e37
# Last command done (1 command done):
#    reword 5760b9c Start article
# Next commands to do (2 remaining commands):
#    fixup a81eec6 Added example of how previously squashed commits
#    fixup 9744a6d Add a bit about interactive rebase use cases
# You are currently editing a commit while rebasing branch 'article/trashy-rebase' on '9328e37'.
#
# Changes to be committed:
#       new file:   articles/019-trash-git-interactive-rebase.md
#
```

We can then change the commit message which will be associated with this commit, e.g.

```
[#XX] Write article about interactive rebase

# Please enter the commit message for your changes. Lines starting
# with '#' will be ignored, and an empty message aborts the commit.
#
```

and then

```
$ git log

commit 00a6031d03d707e316c00c2b1716e88ec212c239 (HEAD -> article/trashy-rebase)
Author: Peter LeVasseur <plevasseur@gmail.com>
Date:   Wed Sep 4 13:54:29 2024 -0400

    [#XX] Write article about interactive rebase

commit 9328e37d8b3e063f4e01e94b78501e61e7fda033 (origin/main, origin/HEAD)
Merge: 69247df f57d138
Author: Pete LeVasseur <plevasseur@gmail.com>
Date:   Fri Aug 30 21:58:16 2024 -0400

    Merge pull request #36 from PLeVasseur/article/gracefully-cpp-interop-drop-impl.md
    
    Article/gracefully cpp interop drop impl.md
```

#### Reorder commits to combine logical units, spin off separate PRs

What happens more often than not for me is rapidly context switching between different parts of the code base, tests, and so on as I'm iterating on how to implement a feature. I find this to be a better fit for my brain as that make it possible do _vertical slices_ through the codebase.

For example, there's three fairly related features I am looking to implement for [up-transport-vsomeip-rust](https://github.com/eclipse-uprotocol/up-transport-vsomeip-rust):

* [Either handle SOME/IP as a monolith authority_name or consider how to accommodate N number of mechatronics devices](https://github.com/eclipse-uprotocol/up-transport-vsomeip-rust/issues/9)
* [Add tests showing functional across two different "devices"](https://github.com/eclipse-uprotocol/up-transport-vsomeip-rust/issues/10)
* [Remove support for multiple vsomeip applications](https://github.com/eclipse-uprotocol/up-transport-vsomeip-rust/issues/21)

Because these are all fairly related, my strategy here would be to implement the latter feature first, followed by the first, then finally the middle one, which is more of a set of end to end tests.

However, I'd like to get all of it implemented before I finally open three PRs implementing each in turn.

The strategy I'd take here is to for each commit that I make, make sure to tag them with which issue they're implementing, e.g.

```bash
git commit -m "21: Switched to use services entry and single application per UPTransportVsomeip instance"
```

This way when I get a little further along and have finished issues 21 and 9, but find an issue implementing the end to end tests in 10, I can still have some commit mixed among them such as this:

```bash
git commit -m "9: fixing instance IDs so that end to end tests will work in 10"
```

That way later when we `git rebase --interactive xxx` it'll look something like this:

```
pick 5760b9c 21: Switched to use services entry and single application per UPTransportVsomeip instance 
pick a81eec6 9: Handle N number of mechatronics service instances
pick 9744a6d 10: Start implementing end to end tests
pick 0d55162 10: Added client_service e2e test betwen docker images
pick 069c45c 9: fixing instance IDs so that end to end tests will work in 10
pick 08d6493 10: finish e2e tests for point_to_point and publisher_subscriber
```

So we can do the following to reorder and combine commits. The next steps I'm showing one by one, but we will save and quit _once_ when finished.

We pull commit `069c45c` up underneath `a81eec6`

```
pick 5760b9c 21: Switched to use services entry and single application per UPTransportVsomeip instance 
pick a81eec6 9: Handle N number of mechatronics service instances
pick 069c45c 9: fixing instance IDs so that end to end tests will work in 10
pick 9744a6d 10: Start implementing end to end tests
pick 0d55162 10: Added client_service e2e test betwen docker images
pick 08d6493 10: finish e2e tests for point_to_point and publisher_subscriber
```

We mark `069c45c` as fixup so it'll get squashed into `a81eec6`

```
pick 5760b9c 21: Switched to use services entry and single application per UPTransportVsomeip instance 
pick a81eec6 9: Handle N number of mechatronics service instances
fixup 069c45c 9: fixing instance IDs so that end to end tests will work in 10
pick 9744a6d 10: Start implementing end to end tests
pick 0d55162 10: Added client_service e2e test betwen docker images
pick 08d6493 10: finish e2e tests for point_to_point and publisher_subscriber
```

We then want to as previously "squash" the other commits dedicated to issue 10 using fixup and mark the first 10 commit as reword so that we can more clearly and completely note what we did.

```
pick 5760b9c 21: Switched to use services entry and single application per UPTransportVsomeip instance 
pick a81eec6 9: Handle N number of mechatronics service instances
fixup 069c45c 9: fixing instance IDs so that end to end tests will work in 10
reword 9744a6d 10: Start implementing end to end tests
fixup 0d55162 10: Added client_service e2e test betwen docker images
fixup 08d6493 10: finish e2e tests for point_to_point and publisher_subscriber
```

We would then have three commits on this branch so that we can then spin each one off into its own feature branch that builds one upon the other.

## And that's it!

I'm still getting started with how to use git's interactive rebase, but from what I can see so far, it's a powerful and ergonomic way to write some trashy commits with a little forethought and reassemble a reasonable commit history.

