#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <assert.h>
#include <sys/types.h>
#include <fcntl.h>

int main() {
    pid_t child = fork();
    if (child < 0) {
        perror("Failed to fork");
        exit(EXIT_FAILURE);
    }
    if (child == 0) {
        // in child process
        if (setsid() < 0) { // detach from terminal
            perror("Failed to create new session");
            exit(EXIT_FAILURE);
        }
        // redirect output to /tmp/teld-worker.log
        int fd = open("/tmp/teld-worker.log", O_WRONLY | O_CREAT | O_APPEND, 0644);
        int nullfd = open("/dev/null", O_RDONLY);
        if (fd < 0) {
            perror("Failed to open log file");
            exit(EXIT_FAILURE);
        }
        if (nullfd < 0) {
            perror("Failed to open /dev/null");
            exit(EXIT_FAILURE);
        }
        dup2(fd, STDOUT_FILENO);
        dup2(fd, STDERR_FILENO);
        dup2(nullfd, STDIN_FILENO);
        close(fd);
        close(nullfd);

        printf("Starting teld-worker...\n");
        fflush(stdout);

        // store pid in /var/lib/teld/teld-worker.pid
        FILE *pid_file = fopen("/var/lib/teld/teld-worker.pid", "w");
        if (pid_file) {
            fprintf(pid_file, "%d\n", getpid());
            fclose(pid_file);
        } else {
            perror("Failed to write PID file");
        }

        execl("/usr/local/bin/teld-worker", "teld-worker", NULL);
        // if execl returns, it means it failed
        perror("Failed to launch teld-worker");
        exit(EXIT_FAILURE);
    } else {
        printf("Launched teld-worker with PID %d\n", child);
        fflush(stdout);
        exit(EXIT_SUCCESS);
    }
}