/* Independent SDK oracle; compiled only for verification, never shipped. */
#include <errno.h>
#include <fcntl.h>
#include <signal.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/ioctl.h>
#include <sys/wait.h>
#include <unistd.h>

#define TYPE(name, type) do { value(name ".size", sizeof(type)); value(name ".align", _Alignof(type)); } while (0)
#define VALUE(name) value(#name, (unsigned long long)(name))
#define SIGNATURE(name, type) _Static_assert(__builtin_types_compatible_p(__typeof__(&(name)), type), #name " prototype")
SIGNATURE(ioctl, int (*)(int, unsigned long, ...));
SIGNATURE(grantpt, int (*)(int));
SIGNATURE(unlockpt, int (*)(int));
SIGNATURE(setsid, int32_t (*)(void));
SIGNATURE(sigemptyset, int (*)(uint32_t *));
SIGNATURE(sigprocmask, int (*)(int, const uint32_t *, uint32_t *));
SIGNATURE(sigaction, int (*)(int, const struct sigaction *, struct sigaction *));
SIGNATURE(waitpid, int32_t (*)(int32_t, int *, int));
_Static_assert(__builtin_types_compatible_p(pid_t, int32_t), "signed pid_t");
_Static_assert(__builtin_types_compatible_p(sigset_t, uint32_t), "unsigned sigset_t");
_Static_assert(__builtin_types_compatible_p(__typeof__(((struct winsize *)0)->ws_row), unsigned short), "winsize field");
_Static_assert(__builtin_types_compatible_p(__typeof__(((struct winsize *)0)->ws_col), unsigned short), "winsize field");
_Static_assert(__builtin_types_compatible_p(__typeof__(((struct winsize *)0)->ws_xpixel), unsigned short), "winsize field");
_Static_assert(__builtin_types_compatible_p(__typeof__(((struct winsize *)0)->ws_ypixel), unsigned short), "winsize field");

static const int signals[] = { SIGHUP, SIGINT, SIGQUIT, SIGTERM, SIGPIPE, SIGCHLD, SIGWINCH, SIGTSTP, SIGTTIN, SIGTTOU };
static void value(const char *name, unsigned long long number) { printf("%s=%llu\n", name, number); }

static int check_inherited(void) {
    sigset_t mask;
    if (sigprocmask(SIG_SETMASK, NULL, &mask) != 0) return 20;
    if (sigismember(&mask, SIGHUP) != 1 || sigismember(&mask, SIGTERM) != 1 || sigismember(&mask, SIGWINCH) != 1) return 21;
    const int ignored[] = { SIGHUP, SIGWINCH, SIGTTOU };
    for (size_t i = 0; i < sizeof(ignored) / sizeof(ignored[0]); ++i) {
        struct sigaction action;
        if (sigaction(ignored[i], NULL, &action) != 0 || action.sa_handler != SIG_IGN) return 22;
    }
    return 0;
}

static int check_child(void) {
    sigset_t mask;
    if (sigprocmask(SIG_SETMASK, NULL, &mask) != 0) return 10;
    for (int s = 1; s < NSIG; ++s) if (sigismember(&mask, s) != 0) return 11;
    for (size_t i = 0; i < sizeof(signals) / sizeof(signals[0]); ++i) {
        struct sigaction action;
        if (sigaction(signals[i], NULL, &action) != 0 || action.sa_handler != SIG_DFL) return 12;
    }
    if (!isatty(0) || !isatty(1) || !isatty(2)) return 13;
    if (getsid(0) != getpid() || tcgetpgrp(0) != getpgrp()) return 14;
    puts("SIG OK");
    return 0;
}

int main(int argc, char **argv) {
    if (argc == 2 && strcmp(argv[1], "--inherited") == 0) return check_inherited();
    if (argc == 2 && strcmp(argv[1], "--child") == 0) return check_child();
    TYPE("int", int); TYPE("ulong", unsigned long); TYPE("pid", pid_t); TYPE("sigset", sigset_t);
    TYPE("winsize", struct winsize); TYPE("sigaction", struct sigaction);
    value("winsize.rows", offsetof(struct winsize, ws_row));
    value("winsize.columns", offsetof(struct winsize, ws_col));
    value("winsize.xpixel", offsetof(struct winsize, ws_xpixel));
    value("winsize.ypixel", offsetof(struct winsize, ws_ypixel));
    value("sigaction.handler", offsetof(struct sigaction, sa_handler));
    value("sigaction.mask", offsetof(struct sigaction, sa_mask));
    value("sigaction.flags", offsetof(struct sigaction, sa_flags));
    sigset_t empty;
    if (sigemptyset(&empty) != 0) return 1;
    value("empty_mask", empty);
    value("default_handler_is_null", SIG_DFL == NULL);
    VALUE(O_NOCTTY); VALUE(O_NONBLOCK); VALUE(TIOCSWINSZ); VALUE(TIOCPTYGNAME); VALUE(TIOCSCTTY);
    VALUE(SIG_SETMASK); VALUE(WNOHANG); VALUE(ECHILD);
    for (size_t i = 0; i < sizeof(signals) / sizeof(signals[0]); ++i) printf("signal.%zu=%d\n", i, signals[i]);
    return 0;
}
