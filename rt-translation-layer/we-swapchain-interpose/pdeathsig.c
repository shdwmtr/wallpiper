#include <errno.h>
#include <signal.h>
#include <sys/prctl.h>
#include <unistd.h>

#include "log.h"

__attribute__((constructor)) static void wp_register_pdeathsig(void)
{
    if (prctl(PR_SET_PDEATHSIG, SIGKILL) != 0) {
        wp_log("failed to register PR_SET_PDEATHSIG: errno=%d", errno);
        return;
    }

    if (getppid() == 1) {
        wp_log("parent already gone by the time PR_SET_PDEATHSIG was registered, self-terminating");
        raise(SIGKILL);
        return;
    }

    wp_log("registered PR_SET_PDEATHSIG(SIGKILL), pid=%d ppid=%d", (int)getpid(), (int)getppid());
}
