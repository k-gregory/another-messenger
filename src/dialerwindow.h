#pragma once

#include <QMainWindow>
#include <QUdpSocket>
#include <QPushButton>
#include <QLineEdit>
#include <QLabel>
#include <QGridLayout>
#include <QListWidget>

class DialerWindow : public QMainWindow{
    Q_OBJECT

public:
    explicit DialerWindow(quint16 port, QWidget* parent = nullptr);

private slots:
    void dataArrived();
    void dialInitiated();

private:
    void initUi();
    QUdpSocket *socket;

    QGridLayout *layout;

    QLabel *ipLabel;
    QLineEdit *ipEdit;

    QLabel *portLabel;
    QLineEdit *portEdit;

    QPushButton *dialBtn;

    QListWidget *chatLog;
    QLineEdit *msgEdit;
};
