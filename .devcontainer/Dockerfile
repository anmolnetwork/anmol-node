FROM paritytech/ci-linux:ba887019-20210411

ARG USERNAME=anmol
ARG USER_UID=1000
ARG USER_GID=$USER_UID

RUN groupadd --gid ${USER_GID} ${USERNAME} \
    && useradd --uid ${USER_UID} --gid ${USER_GID} -s /bin/bash -m ${USERNAME} \
    && apt-get update \
    && apt-get install -y sudo \
    && echo ${USERNAME} ALL=\(root\) NOPASSWD:ALL > /etc/sudoers.d/${USERNAME} \
    && chmod 0440 /etc/sudoers.d/${USERNAME} \
    && mkdir -p /builds/target /home/${USERNAME}/.vscode-server/extensions \
    && chown -R ${USERNAME}:${USERNAME} /builds /home/${USERNAME}/.vscode-server

USER ${USERNAME}:${USERNAME}

VOLUME [ "/builds/target", "/home/${USERNAME}/.vscode-server/extensions" ]
